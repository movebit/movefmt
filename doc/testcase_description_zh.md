# 用例说明
## 测试用例路径:
https://github.com/movebit/movefmt/tests/formatter

## 规则
1.宽度合适(90个字符宽度),缩进合理(2个空格),布局优雅
2.提供尽量少的配置,比如提供缩进用2个或4个空格
3.最大宽度内部强制90个字符
4.内部强制语句块的左'{'不另写单独一行
5.if/while等关键词与其后面的'(' 留一个空格
6.多元操作符前后各一个空格
7.参考业界工具，单行里有块注释过长，不对注释拆行
8.行注释如果在代码尾部，则行注释符前和代码保持两个空格
```rust
    let y: u64 = 100;  // comment(2 space before '//')
```
9.注释符后留一个空格
```rust
    let y: u64 = 100;  // comment(2 space before '//', 1 space after '//')
    /* 1 space after '/*', 1 space before '*/' */
```

## 注释类型:
块注释 -- /**/
行注释 -- // 
文档注释 -- ///


# 分类详解
## 1.表达式
### case1:
同一行多个带分号的语句,尾部有注释.格式化结果应该让尾部注释跟随最有一个分号的语句.
> code snippet from tests/formatter/expr/input1.move
```rust
    let y: u64 = 100; let x: u64 = 0;// Define an unsigned 64-bit integer variable y and assign it a value of 100  
```
效果如下:
> code snippet from tests/formatter/expr/input1.move.fmt
```rust
    let y: u64 = 100;
    let x: u64 = 0;  // Define an unsigned 64-bit integer variable y and assign it a value of 100  
```


### case2:
格式化程序对于 let 表达式的处理.
> code snippet from tests/formatter/expr/input2.move
```rust
let z = if (y <= 10) y = y + 1 else y = 10;  
```

### case3:
格式化程序对于 if/else 结合在同一行,过长,格式化后会拆成两行.
> code snippet from tests/formatter/expr/input3.move
```rust
let z = if (y /*condition check*/ <= /*less than or equal to*/ 10) y = /*assignment*/ y + /*increment*/ 1 else y = /*assignment*/ 10;  
```

### case4:
格式化程序对于 let 和 if/else 结合的处理, 并夹杂复杂注释
> code snippet from tests/formatter/expr/input4.move
```rust
    let /*comment*/z/*comment*/ = if/*comment*/ (/*comment*/y <= /*comment*/10/*comment*/) { // If y is less than or equal to 10  
        y = y + 1; // Increment y by 1  
    }/*comment*/ else /*comment*/{  
        y = 10; // Otherwise, set y to 10  
    };  
```

### case5:
(加减乘除...)表达式

## 2.函数
### case1:
函数名很长,格式化程序要保证不会将函数名拆断
> code snippet from tests/formatter/fun/input1.move
```rust
public fun test_long_fun_name_lllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll(v: u64): SomeOtherStruct { SomeOtherStruct { some_field: v } }
```

### case2:
两个函数块之间紧密贴着,无空行
> code snippet from tests/formatter/fun/input2.move
```rust
  // test two fun Close together without any blank lines
  public fun test_long_fun_name_lllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll(v: u64): SomeOtherStruct{
    SomeOtherStruct{some_field: v}
  }
  public fun multi_arg(p1: u64, p2: u64): u64{
    p1 + p2
  }
```

### case3:
两个函数块之间紧密贴着,中间有行注释或块注释
> code snippet from tests/formatter/fun/input3.move
```rust
public fun multi_arg22(p1: u64, p2: u64): u64{
    p1 + p2
  }/* test two fun Close together without any blank lines, and here is a BlockComment */public fun multi_arg22(p1: u64, p2: u64): u64{
    p1 + p2
  }
```

### case4:
函数返回类型前后有注释
> code snippet from tests/formatter/fun/input4.move
```rust
  public fun multi_arg(p1:u64,p2:u64):/* test comment locate before return type */u64{
    p1 + p2
  }

  public fun multi_arg(p1:u64,p2:u64):u64/* test comment locate after return type */{
    p1 + p2
  }
```

### case5:
函数头有 acquires 语句,且带注释
> code snippet from tests/formatter/fun/input5.move
```rust
  fun acq22(addr: address): u64 acquires SomeStruct/* test comment locate after acquires */{
    let val = borrow_global<SomeStruct>(addr);
    val.some_field
  }

  fun acq33(addr: address): u64 acquires/* test comment locate between acquires */SomeStruct {
    let val = borrow_global<SomeStruct>(addr);
    val.some_field
  }
```

## 3.lambda表达式
### case1:
函数参数为lambda表达式
> code snippet from tests/formatter/lambda/input1.move
```rust
public inline fun inline_apply1(f: |u64|u64, b: u64) : u64 {  
```

### case2:
lambda在while循环体里调用
> code snippet from tests/formatter/lambda/input2.move
```rust
    public inline fun foreach<T>(v: &vector<T>, action: |&T|) {  
        // Loop through the vector and apply the action to each element  
        let i = 0;  
        while (i < vector::length(v)) {  
            action(vector::borrow(v, i));  
            i = i + 1;  
        }  
    }  
```

### case3:
函数体里有lambda表达式的调用
> code snippet from tests/formatter/lambda/input3.move
```rust
        // Apply a lambda function to each element of the vector and update the product variable  
        foreach(&v, |e| product = LambdaTest1::inline_mul(product, *e));  
```

### case4:
lambda嵌套调用
> code snippet from tests/formatter/lambda/input4.move
```rust
    // Public inline function with comments for parameters and return value  
    public inline fun inline_apply2(/* function g */ g: |u64|u64, /* value c */ c: u64) /* returns u64 */ : u64 {  
        // Apply the lambda function g to the result of applying another lambda function to c, and add 2 to the result  
        LambdaTest1::inline_apply1(|z|z, g(LambdaTest1::inline_mul(c, LambdaTest1::inline_apply(|x|x, 3)))) + 2  
    }  
```

### case5:
lambda出现的地方伴随注释
> code snippet from tests/formatter/lambda/input5.move
```rust
    // Public inline function with comments for each parameter and the return value  
    public inline fun inline_apply3(/* lambda function g */ g: |u64|u64, /* value c */ c: u64) /* returns u64 */ : u64 {  
        // Apply the lambda function g to the result of applying another lambda function to c, multiply the result by 3, and add 4 to the result  
        LambdaTest1::inline_apply1(g, LambdaTest1::inline_mul(c, LambdaTest1::inline_apply(|x| { LambdaTest1::inline_apply(|y|y, x) }, 3))) + 4  
    }
```

## 4.列表
### case1:
二维数组,各元素垂直排列,每一个元素前有一行注释
> code snippet from tests/formatter/list/input1.move
```rust
    // Vectorize fee store tier parameters  
    let integrator_fee_store_tiers = vector[  
        // Tier 0 parameters  
        vector[FEE_SHARE_DIVISOR_0,  
               TIER_ACTIVATION_FEE_0,  
               WITHDRAWAL_FEE_0],  
        // Tier 1 parameters  
        vector[FEE_SHARE_DIVISOR_1,  
               TIER_ACTIVATION_FEE_1,  
               WITHDRAWAL_FEE_1],  
        // Tier 2 parameters  
        vector[FEE_SHARE_DIVISOR_2,  
               TIER_ACTIVATION_FEE_2,  
               WITHDRAWAL_FEE_2],  
        // Tier 3 parameters  
        vector[FEE_SHARE_DIVISOR_3,  
               TIER_ACTIVATION_FEE_3,  
               WITHDRAWAL_FEE_3],  
        // Tier 4 parameters  
        vector[FEE_SHARE_DIVISOR_4,  
               TIER_ACTIVATION_FEE_4,  
               WITHDRAWAL_FEE_4],  
        // Tier 5 parameters  
        vector[FEE_SHARE_DIVISOR_5,  
               TIER_ACTIVATION_FEE_5,  
               WITHDRAWAL_FEE_5],  
        // Tier 6 parameters  
        vector[FEE_SHARE_DIVISOR_6,  
               TIER_ACTIVATION_FEE_6,  
               WITHDRAWAL_FEE_6]];  
```

### case2:
数组各元素放在同一行,每个元素前后可能有块注释
> code snippet from tests/formatter/list/input2.move
```rust
    /** Vectorize fee store tier parameters */  
    let integrator_fee_store_tiers = vector[/** Tier 0 parameters */ vector[FEE_SHARE_DIVISOR_0, /** Activation fee for tier 0 */ TIER_ACTIVATION_FEE_0, /** Withdrawal fee for tier 0 */ WITHDRAWAL_FEE_0], /** Tier 1 parameters */ vector[FEE_SHARE_DIVISOR_1, TIER_ACTIVATION_FEE_1, WITHDRAWAL_FEE_1], /** Tier 2 parameters */ vector[FEE_SHARE_DIVISOR_2, TIER_ACTIVATION_FEE_2, WITHDRAWAL_FEE_2], /** Tier 3 parameters */ vector[FEE_SHARE_DIVISOR_3, TIER_ACTIVATION_FEE_3, WITHDRAWAL_FEE_3], /** Tier 4 parameters */ vector[FEE_SHARE_DIVISOR_4, TIER_ACTIVATION_FEE_4, WITHDRAWAL_FEE_4], /** Tier 5 parameters */ vector[FEE_SHARE_DIVISOR_5, TIER_ACTIVATION_FEE_5, WITHDRAWAL_FEE_5], /** Tier 6 parameters */ vector[FEE_SHARE_DIVISOR_6, TIER_ACTIVATION_FEE_6, WITHDRAWAL_FEE_6]];  

```

### case3:
二维数组,各元素垂直排列,每一行的元素末跟注释
> code snippet from tests/formatter/list/input3.move
```rust
    // Define a vector of fee store tiers as a 2D vector  
    let integrator_fee_store_tiers = vector[vector[FEE_SHARE_DIVISOR_0, // Fee share divisor for tier 0  
                                                  TIER_ACTIVATION_FEE_0, // Activation fee for tier 0  
                                                  WITHDRAWAL_FEE_0],      // Withdrawal fee for tier 0  
                                            vector[FEE_SHARE_DIVISOR_1, // Fee share divisor for tier 1  
                                                  TIER_ACTIVATION_FEE_1, // Activation fee for tier 1  
                                                  WITHDRAWAL_FEE_1],      // Withdrawal fee for tier 1  
                                            vector[FEE_SHARE__DIVISOR__2, FEE__SHARE__DIVISOR__2, FEE__SHARE__DIVISOR__2]]; // ... and so on for other tiers  
```

### case4:
二维数组,各元素垂直排列,每一行的元素前后都有块注释
> code snippet from tests/formatter/list/input4.move
```rust
   // Vectorize fee store tier parameters  
    let integrator_fee_store_tiers = vector[  
        // Tier 0 parameters  
        vector[//comment
        FEE_SHARE_DIVISOR_0,  
               TIER_ACTIVATION_FEE_0,  
               WITHDRAWAL_FEE_0],  
        // Tier 1 parameters  
        vector[FEE_SHARE_DIVISOR_1,  
        //comment
               TIER_ACTIVATION_FEE_1,  
               WITHDRAWAL_FEE_1],  
        // Tier 2 parameters  
        vector[FEE_SHARE_DIVISOR_2, 
                //comment 
               TIER_ACTIVATION_FEE_2,  
               WITHDRAWAL_FEE_2],  
        // Tier 3 parameters  
        vector[/*comment*/FEE_SHARE_DIVISOR_3,  
               TIER_ACTIVATION_FEE_3/*comment*/,  
               WITHDRAWAL_FEE_3],  
        // Tier 4 parameters  
        vector[FEE_SHARE_DIVISOR_4,  
               TIER_ACTIVATION_FEE_4, /*comment*/ 
               WITHDRAWAL_FEE_4],  
        // Tier 5 parameters  
        vector[FEE_SHARE_DIVISOR_5,  
               /*comment*/TIER_ACTIVATION_FEE_5,  
               WITHDRAWAL_FEE_5],  
        // Tier 6 parameters  
        vector[FEE_SHARE_DIVISOR_6,  
               TIER_ACTIVATION_FEE_6,  
               WITHDRAWAL_FEE_6]];  /*comment*/
```

### case5:
二维数组,各元素垂直排列,某些行之间有空行
> code snippet from tests/formatter/list/input5.move
```rust
    let integrator_fee_store_tiers = vector[
        vector[FEE_SHARE_DIVISOR_0, TIER_ACTIVATION_FEE_0, WITHDRAWAL_FEE_0],
        
        vector[FEE_SHARE_DIVISOR_1, TIER_ACTIVATION_FEE_1, WITHDRAWAL_FEE_1],
        // ...
        vector[FEE_SHARE_DIVISOR_N, TIER_ACTIVATION_FEE_N, WITHDRAWAL_FEE_N]
    ];
```

## 5.spec_fun
### case1:
has {'apply', 'exists', 'global'}
> code snippet from tests/formatter/spec_fun/input1.move
```rust
apply CapAbortsIf to *<Feature> except spec_delegates;
// ...
exists<CapState<Feature>>(addr)
// ...
global<CapState<Feature>>(addr).delegates
```

### case2:
No blank lines between two functions
> code snippet from tests/formatter/spec_fun/input2.move
```rust
spec fun spec_table_len<K, V>(t: TableWithLength<K, V>): u64 {
            table_with_length::spec_len(t)
        }
        spec fun spec_table_contains<K, V>(t: TableWithLength<K, V>, k: K): bool {
            table_with_length::spec_contains(t, k)
        }
```

### case3:
comment between two functions
> code snippet from tests/formatter/spec_fun/input3.move
```rust
spec fun spec_table_len<K, V>(t: TableWithLength<K, V>): u64 {
            table_with_length::spec_len(t)
        }
// comment
        spec fun spec_table_contains<K, V>(t: TableWithLength<K, V>, k: K): bool {
            table_with_length::spec_contains(t, k)
        }
```

### case4:
fun name too long
> code snippet from tests/formatter/spec_fun/input4.move
```rust
spec singletonlllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll<T: store>(element: T, bucket_size: u64): BigVector<T> {
    ensures length(result) == 1;
    ensures result.bucket_size == bucket_size;
}
```

### case5:
all kinds of comment in spec fun
> code snippet from tests/formatter/spec_fun/input5.move
```rust
    spec fun spec_at<T>(v: BigVector<T>/*comment*/, i: u64): T {
        let bucket = i / v.bucket_size;//comment
        //comment
        let idx =/*comment*/ i % v.bucket_size;
        /// comment
        let v = table_with_length::spec_get(v.buckets, /*comment*/bucket);
        /*comment*/
        v[idx]
    }
```

## 6.spec module
### case1:
has {'pragma', 'aborts_if', 'ensures'}
> code snippet from tests/formatter/spec_module/input1.move
```rust
pragma opaque;
        aborts_if [abstract] false;
        ensures [abstract] result == spec_internal_is_char_boundary(v, i);
```

### case2:
has {'native fun'}
> code snippet from tests/formatter/spec_module/input2.move
```rust
native fun serialize<MoveValue>(v: &MoveValue): vector<u8>;
```

### case3:
has {'requires'}
> code snippet from tests/formatter/spec_module/input3.move
```rust
requires exists<coin::CoinInfo<AptosCoin>>(@aptos_framework);
```

### case4:
There is only one comment and no code in the module block
> code snippet from tests/formatter/spec_module/input4.move
```rust
spec module {/*comment*/} // switch to module documentation context
```

### case5:
has {'use', 'include'}
> code snippet from tests/formatter/spec_module/input5.move
```rust
 use aptos_framework::staking_config;
        pragma verify_duration_estimate = 120; // TODO: set because of timeout (property proved)
    requires chain_status::is_operating();   
        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockAptosSupply;
```

## 7.spec 结构体
### case1:
has ability{'copy', 'drop', 'store'}
> code snippet from tests/formatter/spec_struct/input1.move
```rust
struct String has copy, drop, store {
```

### case2:
has {'invariant'}
> code snippet from tests/formatter/spec_struct/input2.move
```rust
invariant is_valid_char(byte);
```

### case3:
has ability{'copy', 'drop', 'store'} with comment
> code snippet from tests/formatter/spec_struct/input3.move
```rust
struct String has /*comment*/copy, drop/*comment*/, store /*comment*/{
       // comment
       bytes: vector<u8>,// comment

   }
```

### case4:
Struct field has comments
> code snippet from tests/formatter/spec_struct/input4.move
```rust
   /// An ASCII character.
   struct Char has copy, drop, store {
    // comment
       byte: u8,
   }

```

### case5:
Struct ability written on multiple lines
> code snippet from tests/formatter/spec_struct/input5.move
```rust
   /// An ASCII character.
   struct Char has copy,/*comment*/ 
    /*comment*/drop, 
    // comment
    store {
    // comment
       byte: u8,
   }
   spec Char {
    // comment
       invariant is_valid_char(byte);//comment
   }
```

## 8.结构体
### case1:
结构体定义处,每个字段不同位置有注释
> code snippet from tests/formatter/struct/input1.move
```rust
    struct TestStruct1 {  
        // This is field1 comment  
        field1: u64,  
        field2: bool,  
    } 
    struct TestStruct2 { // This is a comment before struct definition 
        field1: u64, // This is a comment for field1  
        field2: bool, // This is a comment for field2  
    }  // This is a comment after struct definition 
    struct TestStruct4<T> {  
        // This is a comment before complex field  
        field: vector<T>, // This is a comment after complex field  
    }  
```

### case2:
带范型的结构体变量作为函数参数
> code snippet from tests/formatter/struct/input2.move
```rust
    // Function using the struct  
    fun use_complex_struct1(s: ComplexStruct1<u64, bool>) {  
        // Function comment  
    }  
```

### case3:
带范型的结构体定义
> code snippet from tests/formatter/struct/input3.move
```rust
    // Struct with comments in various positions  
    struct ComplexStruct1<T, U> {  
        // Field 1 comment  
        field1: vector<U>, // Trailing comment for field1  
        // Field 2 comment  
        field2: bool,  
        // Field 3 comment  
        field3: /* Pre-comment */ SomeOtherStruct<T> /* Post-comment */,  
    } /* Struct footer comment */  
```

### case4:
结构体字段之间有空行或行注释
> code snippet from tests/formatter/struct/input4.move
```rust
    // Struct with nested comments and complex types  
    struct ComplexStruct2<T, U> {  
        
        field1: /* Pre-comment */ vector<T> /* Inline comment */,  
        
        field2: /* Comment before complex type */ SomeGenericStruct<U> /* Comment after complex type */,  
        
        field3: /* Pre-comment */ optional<bool> /* Post-comment */,  
    } // Struct footer comment  
```

### case5:
结构体带has copy, drop, store 等ability
> code snippet from tests/formatter/struct/input5.move
```rust
    // Integrator fee store tier parameters for a given tier.
    struct IntegratorFeeStoreTierParameters has drop, store {
        // Nominal amount divisor for taker quote coin fee.
        fee_share_divisor: u64,
    }
```

## 9.元组
### case1:
函数返回空元组
> code snippet from tests/formatter/tuple/input1.move
```rust
// when no return type is provided, it is assumed to be `()`
fun returs_unit_1() { }

// there is an implicit () value in empty expression blocks
fun returs_unit_2(): () { }

// explicit version of `returs_unit_1` and `returs_unit_2`
fun returs_unit_3(): () { () }
```

### case2:
函数元组不同元素带注释
> code snippet from tests/formatter/tuple/input2.move
```rust
        fun returns_3_values(): (u64, bool, address) {
            // comment
            (0, /*comment*/false/*comment*/, @0x42)// comment
        }
        fun returns_4_values(x: &u64): (&u64, u8, u128, vector<u8>) {            
            (x/*comment*/, 0/*comment*/, /*comment*/1/*comment*/, /*comment*/b"foobar"/*comment*/)
        }
```

### case3:
元组括号里的元素,每个元素占用一行,并夹带注释
> code snippet from tests/formatter/tuple/input3.move
```rust
        fun returns_3_values(): (u64, bool, address) {
            // comment
            (
                // comment
                0, 
            /*comment*/false/*comment*/, 
            @0x42)// comment
        }
```

### case4:
元组定义,被各种类型的表达式赋值
> code snippet from tests/formatter/tuple/input3.move
```rust
            // This line is an example of a unit value being assigned to a variable  
            let () = ();

            // This line is an example of a tuple with multiple types being assigned to variables a, b, c, and d  
            let (a, b, c, d) = (@0x0, 0, false, b"");  
  
            // Reassignment of unit value  
            () = ();  
              
            // Conditional reassignment of tuple values x and y  
            (x, y) = if (cond) (1, 2) else (3, 4);  
              
            // Reassignment of tuple values a, b, c, and d  
            (a, b, c, d) = (@0x1, 1, true, b"1");
```

### case5:
元组定义,被各种类型的表达式赋值,且元素周围带注释
> code snippet from tests/formatter/tuple/input5.move
```rust
        /**  
         * This function returns a tuple containing four values: a reference to a u64 value,  
         * a u8 value, a u128 value, and a vector of u8 values.  
         */  
        fun returns_4_values(x: &u64): /*(&u64, u8, u128, vector<u8>)*/ (&u64, u8, u128, vector<u8>) { (x, /*comment*/0, 1, b"foobar") }  
          
        /**  
         * This function demonstrates various examples using tuples.  
         * It includes assignments to tuple variables and reassignments using conditional statements.  
         */  
        fun examples(cond: bool) {
            // Assignment of tuple values to variables x and y  
            let (x, y): /*(u8, u64)*/ (u8, u64) = (0, /*comment*/1);
        }
```

## 10.use
### case1:
A list consisting of multiple items, with comments after the items
> code snippet from tests/formatter/use/input1.move
```rust
    use aptos_framework::aptos_coin::AptosCoin;
        use aptos_framework::coin::{Self, Coin}/* use */;
                use aptos_std::type_info::{Self/* use_item after */, TypeInfo};
            use econia::resource_account;
        use econia::tablist::{Self, Tablist/* use_item after */};
            use std::signer::address_of;
    use std::vector;
```

### case2:
A list consisting of multiple items, with comments before the items
> code snippet from tests/formatter/use/input2.move
```rust
    use aptos_framework::aptos_coin::AptosCoin;
        use aptos_framework::coin::{Self, Coin};
                use aptos_std::type_info::{/* use_item before */Self, TypeInfo};
            use econia::resource_account;
        use econia::tablist::{Self, /* use_item before */Tablist};
            use std::signer::address_of;
    use std::vector;
```

### case3:
Use items one by one, with block comments on each line
> code snippet from tests/formatter/use/input3.move
```rust
        use aptos_std::type_info::{
            /* use_item before */Self, 
            TypeInfo
        };
    use aptos_framework::coin::{
        Self, 
        /* use_item before */Coin};
```

### case4:
Use items one by one, with inline comments on each line
> code snippet from tests/formatter/use/input4.move
```rust
    use aptos_std::type_info::{
        // use_item
        Self, 
        TypeInfo
    };
use aptos_framework::coin::{
    Self, 
    // use_item
    Coin};
```

### case5:
Multiple blank lines between use statements
> code snippet from tests/formatter/use/input5.move
```rust
    // Multiple blank lines between statements
        use aptos_std::type_info::{
            /* use_item before */Self, 

            TypeInfo
        };



    use aptos_framework::coin::{
        Self, 

        /* use_item before */Coin};
  
```
