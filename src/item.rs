use super::types::*;
use enum_iterator::Sequence;
use move_compiler::shared::Identifier;
use move_compiler::shared::TName;
use move_compiler::{parser::ast::*, shared::*};
use move_core_types::account_address::AccountAddress;

use move_ir_types::location::Loc;
use move_symbol_pool::Symbol;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Clone)]
pub struct ItemStruct {
    pub(crate) name: StructName,
    pub(crate) type_parameters: Vec<StructTypeParameter>,
    pub(crate) type_parameters_ins: Vec<ResolvedType>,
    pub(crate) fields: Vec<(Field, ResolvedType)>, /* TODO If this length is zero,maybe a native. */
    pub(crate) is_test: bool,
    pub(crate) addr: AccountAddress,
    pub(crate) module_name: Symbol,
}

impl ItemStruct {
    pub(crate) fn to_struct_ref(&self) -> ItemStructNameRef {
        ItemStructNameRef {
            addr: self.addr,
            module_name: self.module_name,
            name: self.name,
            type_parameters: self.type_parameters.clone(),
            is_test: self.is_test,
        }
    }
    pub(crate) fn find_filed_by_name(&self, name: Symbol) -> Option<&(Field, ResolvedType)> {
        for f in self.fields.iter() {
            if f.0 .0.value.as_str() == name.as_str() {
                return Some(f);
            }
        }
        None
    }
    pub(crate) fn all_fields(&self) -> HashMap<Symbol, (Name, ResolvedType)> {
        let mut m = HashMap::new();
        self.fields.iter().for_each(|f| {
            m.insert(f.0 .0.value, (f.0 .0.clone(), f.1.clone()));
        });
        m
    }
}

impl ItemStruct {
    pub(crate) fn bind_type_parameter(
        &mut self,
        types: Option<&HashMap<Symbol, ResolvedType>>, //  types maybe infered from somewhere or use sepcified.
    ) {
        let mut m = HashMap::new();

        debug_assert!(
            self.type_parameters_ins.len() == 0
                || self.type_parameters_ins.len() == self.type_parameters.len()
        );
        let types = if let Some(types) = types {
            self.type_parameters.iter().for_each(|x| {
                self.type_parameters_ins.push(
                    types
                        .get(&x.name.value)
                        .map(|x| x.clone())
                        .unwrap_or_default(),
                )
            });
            types
        } else {
            for (k, v) in self
                .type_parameters
                .iter()
                .zip(self.type_parameters_ins.iter())
            {
                m.insert(k.name.value, v.clone());
            }
            &m
        };
        self.fields
            .iter_mut()
            .for_each(|f| f.1.bind_type_parameter(types));
    }
}

impl std::fmt::Display for ItemStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.value().as_str())
    }
}

#[derive(Clone)]
pub enum Item {
    Parameter(Var, ResolvedType),
    Const(ItemConst),
    Var {
        var: Var,
        ty: ResolvedType,
        lambda: Option<LambdaExp>,
        has_decl_ty: bool,
    },
    Field(Field, ResolvedType),
    Struct(ItemStruct),
    StructNameRef(ItemStructNameRef),
    Fun(ItemFun),
    MoveBuildInFun(MoveBuildInFun),
    SpecBuildInFun(SpecBuildInFun),
    SpecConst(ItemConst),
    /// build in types.
    BuildInType(BuildInType),
    TParam(Name, Vec<Ability>),
    SpecSchema(Name, HashMap<Symbol, (Name, ResolvedType)>),
    /// a module name in 0x1111::module_name
    ModuleName(ItemModuleName),
    Use(Vec<ItemUse>),
    Dummy,
}

#[derive(Clone)]
pub struct LambdaExp {
    pub(crate) bind_list: BindList,
    pub(crate) exp: Exp,
}

#[derive(Clone)]
pub enum ItemUse {
    Module(ItemUseModule),
    Item(ItemUseItem),
}

#[derive(Clone)]
pub struct ItemUseModule {
    pub(crate) module_ident: ModuleIdent,         // 0x111::xxxx
    pub(crate) alias: Option<ModuleName>,         // alias
    pub(crate) s: Option<Name>,                   // Option Self
    #[allow(dead_code)]
    pub(crate) is_test: bool,
}

#[derive(Clone)]
pub struct ItemUseItem {
    pub(crate) module_ident: ModuleIdent, /* access name */
    pub(crate) name: Name,
    pub(crate) alias: Option<Name>, /* alias  */
    #[allow(dead_code)]
    pub(crate) is_test: bool,
}

#[derive(Clone)]
pub struct ItemModuleName {
    pub(crate) name: ModuleName,
    pub(crate) is_test: bool,
}

#[derive(Clone)]
pub struct ItemStructNameRef {
    pub(crate) addr: AccountAddress,
    pub(crate) module_name: Symbol,
    pub(crate) name: StructName,
    pub(crate) type_parameters: Vec<StructTypeParameter>,
    pub(crate) is_test: bool,
}

#[derive(Clone)]
pub struct ItemFun {
    pub(crate) name: FunctionName,
    pub(crate) type_parameters: Vec<(Name, Vec<Ability>)>,
    pub(crate) parameters: Vec<(Var, ResolvedType)>,
    pub(crate) ret_type: Box<ResolvedType>,
    pub(crate) ret_type_unresolved: Type,
    pub(crate) is_spec: bool,
    pub(crate) vis: Visibility,
    pub(crate) is_test: AttrTest,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AttrTest {
    No,
    Test,
    TestOnly,
}

impl AttrTest {
    pub(crate) fn is_test(self) -> bool {
        self == Self::Test || self == Self::TestOnly
    }
}

impl std::fmt::Display for ItemFun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fun {}", self.name.value().as_str())?;
        if self.type_parameters.len() > 0 {
            write!(f, "<")?;
            let last = self.type_parameters.len() - 1;
            for (index, (name, _)) in self.type_parameters.iter().enumerate() {
                write!(
                    f,
                    "{}{}",
                    name.value.as_str(),
                    if index < last { "," } else { "" }
                )?;
            }
            write!(f, ">")?;
        }
        write!(f, "(")?;
        if self.parameters.len() > 0 {
            let last = self.parameters.len() - 1;
            for (index, (name, t)) in self.parameters.iter().enumerate() {
                write!(
                    f,
                    "{}:{}{}",
                    name.value().as_str(),
                    t,
                    if index < last { "," } else { "" }
                )?;
            }
        }

        write!(f, ")")?;
        if !self.ret_type.as_ref().is_unit() {
            write!(f, ":{}", self.ret_type.as_ref())?;
        }
        Ok(())
    }
}

impl Item {
    pub(crate) fn is_build_in(&self) -> bool {
        match self {
            Item::BuildInType(_) => true,
            Item::SpecBuildInFun(_) => true,
            Item::MoveBuildInFun(_) => true,
            _ => false,
        }
    }
}

impl Default for Item {
    fn default() -> Self {
        Self::Dummy
    }
}

#[derive(Clone)]
pub struct ItemConst {
    pub(crate) name: ConstantName,
    pub(crate) ty: ResolvedType,
    /// only Const have this field,SpecConst ignore this field.
    pub(crate) is_test: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum MacroCall {
    Assert,
}

impl MacroCall {
    pub(crate) fn from_chain(chain: &NameAccessChain) -> Option<Self> {
        match &chain.value {
            NameAccessChain_::One(name) => return Self::from_symbol(name.value),
            NameAccessChain_::Two(_, _) => return None,
            NameAccessChain_::Three(_, _) => return None,
        }
    }
    pub(crate) fn from_symbol(s: Symbol) -> Option<Self> {
        match s.as_str() {
            "assert" => Some(Self::Assert),
            _ => return None,
        }
    }
    pub(crate) fn to_static_str(self) -> &'static str {
        match self {
            MacroCall::Assert => "assert",
        }
    }
}

impl Default for MacroCall {
    fn default() -> Self {
        Self::Assert
    }
}

/// Get the last name of a access chain.
pub(crate) fn get_name_chain_last_name(x: &NameAccessChain) -> &Name {
    match &x.value {
        move_compiler::parser::ast::NameAccessChain_::One(name)
        | move_compiler::parser::ast::NameAccessChain_::Two(_, name)
        | move_compiler::parser::ast::NameAccessChain_::Three(_, name) => name,
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Parameter(var, t) => {
                write!(f, "{}:{}", var.0.value.as_str(), t)
            }
            Item::ModuleName(ItemModuleName { name, .. }) => {
                write!(f, "{}", name.value().as_str())
            }
            Item::Use(x) => Ok(for x in x.iter() {
                match x {
                    ItemUse::Module(ItemUseModule { module_ident, .. }) => {
                        write!(f, "use {:?} {}", module_ident, "_")?;
                    }
                    ItemUse::Item(ItemUseItem {
                        module_ident,
                        name,
                        alias,
                        ..
                    }) => {
                        write!(
                            f,
                            "use {:?}::{:?} {}",
                            module_ident,
                            name,
                            if let Some(alias) = alias {
                                format!(" as {}", alias.value.as_str())
                            } else {
                                String::from_str("").unwrap()
                            },
                        )?;
                    }
                }
            }),

            Item::Const(ItemConst { name, ty, .. }) => {
                write!(f, "{}:{}", name.0.value.as_str(), ty)
            }
            Item::SpecConst(ItemConst { name, ty, .. }) => {
                write!(f, "{}:{}", name.0.value.as_str(), ty)
            }
            Item::Struct(s) => {
                write!(f, "{}", s)
            }
            Item::StructNameRef(ItemStructNameRef { name, .. }) => {
                write!(f, "{}", name.value().as_str())
            }
            Item::Fun(x) => write!(f, "{}", x),
            Item::BuildInType(x) => {
                write!(f, "{}", x.to_static_str())
            }
            Item::TParam(tname, abilities) => {
                write!(f, "{}:", tname.value.as_str())?;
                for i in 0..abilities.len() {
                    let x = abilities.get(i).unwrap();
                    write!(f, "{:?},", x.value)?;
                }
                std::result::Result::Ok(())
            }
            Item::Var { var, ty, .. } => {
                write!(f, "{}:{}", var.0.value.as_str(), ty)
            }
            Item::Field(x, ty) => {
                write!(f, "{}:{}", x.0.value.as_str(), ty)
            }
            Item::Dummy => {
                write!(f, "dummy")
            }
            Item::SpecSchema(name, _) => {
                write!(f, "{}", name.value.as_str())
            }
            Item::MoveBuildInFun(x) => write!(f, "move_build_in_fun {}", x.to_static_str()),
            Item::SpecBuildInFun(x) => write!(f, "spec_build_in_fun {}", x.to_static_str()),
        }
    }
}

#[derive(Clone)]
pub enum Access {
    ApplyType(NameAccessChain, Option<ModuleName>, Box<ResolvedType>),
    ExprVar(Var, Box<Item>),
    ExprAccessChain(
        NameAccessChain,
        Box<Item>, /* The item that you want to access.  */
    ),
    // Maybe the same as ExprName.
    // TODO   @XXX 目前知道的可以在Move.toml中定义
    // 可以在源代码中定义吗?
    ExprAddressName(Name),
    AccessFiled(AccessFiled),
    ///////////////
    /// key words
    KeyWords(&'static str),
    /////////////////
    /// Marco call
    MacroCall(MacroCall, NameAccessChain),
    Friend(NameAccessChain, ModuleName),

    ApplySchemaTo(
        NameAccessChain, // Apply a schema to a item.
        Box<Item>,
    ),
    IncludeSchema(NameAccessChain, Box<Item>),
    PragmaProperty(PragmaProperty),
    SpecFor(Name, Box<Item>),
}

#[derive(Clone)]
pub struct AccessFiled {
    pub(crate) from: Field,
    pub(crate) to: Field,
    pub(crate) ty: ResolvedType,
    pub(crate) all_fields: HashMap<Symbol, (Name, ResolvedType)>,
    /// When dealing with below syntax can have this.
    /// ```move
    /// let x = XXX {x}
    ///```
    /// x is alas a field and a expr.
    /// and a expr can link to a item.
    pub(crate) item: Option<Item>,
    /// Does this field access contains a ref
    /// like &xxx.yyy
    pub(crate) has_ref: Option<bool>,
}

#[derive(Clone)]
pub enum ItemOrAccess {
    Item(Item),
    Access(Access),
}

impl Into<Item> for ItemOrAccess {
    fn into(self) -> Item {
        match self {
            Self::Item(x) => x,
            _ => unreachable!(),
        }
    }
}

impl Into<Access> for ItemOrAccess {
    fn into(self) -> Access {
        match self {
            Self::Access(x) => x,
            _ => unreachable!(),
        }
    }
}

// impl std::fmt::Display for ItemOrAccess {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Access(a) => a.fmt(f),
//             Self::Item(x) => x.fmt(f),
//         }
//     }
// }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum MoveBuildInFun {
    MoveTo,
    MoveFrom,
    BorrowGlobalMut,
    BorrowGlobal,
    Exits,
}

impl MoveBuildInFun {
    pub(crate) fn to_static_str(self) -> &'static str {
        match self {
            MoveBuildInFun::MoveTo => "move_to",
            MoveBuildInFun::MoveFrom => "move_from",
            MoveBuildInFun::BorrowGlobalMut => "borrow_global_mut",
            MoveBuildInFun::BorrowGlobal => "borrow_global",
            MoveBuildInFun::Exits => "exists",
        }
    }
}

impl std::fmt::Display for MoveBuildInFun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_static_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Sequence)]
pub enum SpecBuildInFun {
    Exists,
    Global,
    Len,
    Update,
    Vec,
    Concat,
    Contains,
    IndexOf,
    Range,
    InRange,
    UpdateField,
    Old,
    TRACE,
}

impl SpecBuildInFun {
    pub(crate) fn to_static_str(self) -> &'static str {
        match self {
            Self::Exists => "exists",
            Self::Global => "global",
            Self::Len => "len",
            Self::Update => "update",
            Self::Vec => "vec",
            Self::Concat => "concat",
            Self::Contains => "contains",
            Self::IndexOf => "index_of",
            Self::Range => "range",
            Self::InRange => "in_range",
            Self::UpdateField => "update_field",
            Self::Old => "old",
            Self::TRACE => "TRACE",
        }
    }

    pub(crate) fn to_notice(self) -> &'static str {
        match self {
            SpecBuildInFun::Exists => {
                r#"exists<T>(address): bool 
Returns true if the resource T exists at address.
                "#
            }
            SpecBuildInFun::Global => {
                r#"global<T>(address): 
T returns the resource value at address."#
            }
            SpecBuildInFun::Len => {
                r#"len<T>(vector<T>): num 
Returns the length of the vector."#
            }
            SpecBuildInFun::Update => {
                r#"update<T>(vector<T>, num, T>): vector<T> 
Returns a new vector with the element replaced at the given index."#
            }
            SpecBuildInFun::Vec => {
                r#"vec<T>(): vector<T> 
Returns an empty vector."#
            }
            SpecBuildInFun::Concat => {
                r#"concat<T>(vector<T>, vector<T>): vector<T> 
Returns the concatenation of the parameters."#
            }
            SpecBuildInFun::Contains => {
                r#"contains<T>(vector<T>, T): bool 
Returns true if element is in vector."#
            }
            SpecBuildInFun::IndexOf => {
                r#"index_of<T>(vector<T>, T): num 
Returns the index of the element in the vector, or the length of the vector if it does not contain it."#
            }
            SpecBuildInFun::Range => {
                r#"range<T>(vector<T>): range 
Returns the index range of the vector."#
            }
            SpecBuildInFun::InRange => {
                r#"in_range<T>(vector<T>, num): bool 
Returns true if the number is in the index range of the vector."#
            }
            SpecBuildInFun::UpdateField => {
                r#"update_field(S, F, T): S 
Updates a field in a struct, preserving the values of other fields, where S is some struct, F the name of a field in S, and T a value for this field."#
            }
            SpecBuildInFun::Old => {
                r#"old(T): T 
T delivers the value of the passed argument at point of entry into a Move function. This is allowed in ensures post-conditions, inline spec blocks (with additional restrictions), and certain forms of invariants, as discussed later."#
            }
            SpecBuildInFun::TRACE => {
                r#"TRACE(T): T
T is semantically the identity function and causes visualization of the argument's value in error messages created by the prover.
            "#
            }
        }
    }
}

impl std::fmt::Display for SpecBuildInFun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_static_str())
    }
}
