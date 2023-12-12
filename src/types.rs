use super::item::*;
use crate::item::ItemFun;
use enum_iterator::Sequence;
// use move_command_line_common::files::FileHash;
use move_compiler::{
    parser::ast::*,
    shared::*,
};
// use move_ir_types::location::{Loc, Spanned};
use move_symbol_pool::Symbol;
use std::collections::HashMap;
use std::fmt::Debug;
// use std::vec;

#[derive(Clone)]
pub enum ResolvedType {
    UnKnown,
    Struct(ItemStructNameRef, Vec<ResolvedType>),
    BuildInType(BuildInType),
    /// T : drop
    TParam(Name, Vec<Ability>),

    /// & mut ...
    Ref(bool, Box<ResolvedType>),
    /// ()
    Unit,
    /// (t1, t2, ... , tn)
    /// Used for return values and expression blocks
    Multiple(Vec<ResolvedType>),
    Fun(ItemFun),
    Vec(Box<ResolvedType>),

    Lambda {
        args: Vec<ResolvedType>,
        ret_ty: Box<ResolvedType>,
    },

    /// Spec type
    Range,
}

impl Default for ResolvedType {
    fn default() -> Self {
        Self::UnKnown
    }
}

impl ResolvedType {
    pub(crate) fn is_unit(&self) -> bool {
        match self {
            ResolvedType::Unit => true,
            _ => false,
        }
    }

    /// bind type parameter to concrete type
    pub(crate) fn bind_type_parameter(&mut self, types: &HashMap<Symbol, ResolvedType>) {
        match self {
            ResolvedType::UnKnown => {}
            ResolvedType::BuildInType(_) => {}
            ResolvedType::TParam(name, _) => {
                if let Some(x) = types.get(&name.value) {
                    *self = x.clone();
                }
            }
            ResolvedType::Ref(_, ref mut b) => {
                b.as_mut().bind_type_parameter(types);
            }
            ResolvedType::Unit => {}
            ResolvedType::Multiple(ref mut xs) => {
                for i in 0..xs.len() {
                    let t = xs.get_mut(i).unwrap();
                    t.bind_type_parameter(types);
                }
            }
            ResolvedType::Fun(x) => {
                let xs = &mut x.parameters;
                for i in 0..xs.len() {
                    let t = xs.get_mut(i).unwrap();
                    t.1.bind_type_parameter(types);
                }
                x.ret_type.as_mut().bind_type_parameter(types);
            }
            ResolvedType::Vec(ref mut b) => {
                b.as_mut().bind_type_parameter(types);
            }

            ResolvedType::Struct(_, ts) => {
                for index in 0..ts.len() {
                    ts.get_mut(index).unwrap().bind_type_parameter(types);
                }
            }
            ResolvedType::Range => {}
            ResolvedType::Lambda { args, ret_ty } => {
                for a in args.iter_mut() {
                    a.bind_type_parameter(types);
                }
                ret_ty.bind_type_parameter(types);
            }
        }
    }
}

#[derive(Clone, Debug, Copy, Sequence)]
pub enum BuildInType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Address,
    /// A number type from literal.
    /// Could be u8 and ... depend on How it is used.
    NumType,
    /// https://move-book.com/advanced-topics/managing-collections-with-vectors.html?highlight=STring#hex-and-bytestring-literal-for-inline-vector-definitions
    /// alias for vector<u8>
    String,
    Signer,
}

impl BuildInType {
    pub(crate) fn to_static_str(self) -> &'static str {
        match self {
            BuildInType::U8 => "u8",
            BuildInType::U16 => "u16",
            BuildInType::U32 => "u32",
            BuildInType::U64 => "u64",
            BuildInType::U128 => "u128",
            BuildInType::U256 => "u256",
            BuildInType::Bool => "bool",
            BuildInType::Address => "address",
            BuildInType::Signer => "signer",
            BuildInType::String => "vector<u8>",
            BuildInType::NumType => "u256",
        }
    }
}

impl std::fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::BuildInType(x) => write!(f, "{}", x.to_static_str()),
            ResolvedType::TParam(name, _) => {
                write!(f, "{}", name.value.as_str())
            }
            ResolvedType::Ref(is_mut, ty) => {
                write!(f, "&{}{}", if *is_mut { "mut " } else { "" }, ty.as_ref())
            }
            ResolvedType::Unit => write!(f, "()"),
            ResolvedType::Multiple(m) => {
                write!(f, "(")?;
                for i in 0..m.len() {
                    let t = m.get(i).unwrap();
                    write!(f, "{}{}", if i == m.len() - 1 { "" } else { "," }, t)?;
                }
                write!(f, ")")
            }
            ResolvedType::Fun(x) => {
                write!(f, "{}", x)
            }
            ResolvedType::Vec(ty) => {
                write!(f, "vector<<{}>>", ty.as_ref())
            }
            ResolvedType::Range => {
                write!(f, "range(n..m)")
            }
            ResolvedType::Lambda { args, ret_ty } => {
                write!(f, "|")?;
                if args.len() > 0 {
                    let last_index = args.len() - 1;
                    for (index, a) in args.iter().enumerate() {
                        write!(f, "{}", a)?;
                        if index != last_index {
                            write!(f, ",")?;
                        }
                    }
                }
                write!(f, "|")?;
                if matches!(ret_ty.as_ref(), ResolvedType::Unit) == false {
                    write!(f, ":")?;
                    write!(f, "{}", ret_ty)
                } else {
                    Ok(())
                }
            }
            _ => write!(f, "unknown"),
        }
    }
}
