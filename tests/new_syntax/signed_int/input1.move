module test {
public fun test_i128(): i128 {
let a = 1u128; // type mismatch in `res5 + a`: expected i128, actual u128
let b = 170141183460469231731687303715884105728; // interpreted as a possible u128|u256
let c = 170141183460469231731687303715884105728i128; // type mismatch in `res7 + c`, expected u128|u256, actual i128
let d = - 170141183460469231731687303715884105729; // no type can be inffered
let e = - 170141183460469231731687303715884105729i128; // constant does not fit into i128
let res1 = V1_128 + V2_128;
let res2 = res1 + V3_128;
let res3 = res2 + V4_128;
let res4 = res3 + V5_128;
let res5 = res4 + V6_128;
let res6 = res5 + a;
let res7 = res6 + b;
let res8 = res7 + c;
let res9 = res8 + d;
let res10 = res9 + e;
res10
}

    fun test_neg1(x: i64, y: i64): i64 {
        (- x) + y - (- x) * (- y) / (- x) % (- y)
    }

    fun test_neg2(x: i128, y: i128): i128 {
        (- x) + y - (- x) * (- y) / (- x) % (- y)
    }
}