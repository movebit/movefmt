module test_enum {
    enum Shape {
    Circle{radius: u64},
    Rectangle{width: u64, height: u64}
}

// enum variant may has no fields
enum Color {
  Red, Blue, Green
}

// enum types can have abilities
enum Color2 has copy, drop, store, key { Red, Blue, Green }

enum VersionedData has key {
  V1{name: String}
  V2{name: String, age: u64}
}

// enum types can be generic and take positional arguments
enum Result<T> has copy, drop, store {
  Err(u64),
  Ok(T)
}


fun f(data: VersionedData) {
    // constructed similarly to a struct value
    let s: String;
    let data = VersionedData::V1{name: s};

    // If the enum variant has no fields, the braces can also be omitted:
let color = Color::Blue;

// the Move compiler can infer the enum type from the context, 
// and the qualification by the type name may be omitted:

  match (data) { V1{..} => {}, _ => {} } // simple variant name OK
}


fun area(self: &Rectangle): u64 {
    match (self) {
        Circle{radius}           => mul_with_pi(*radius * *radius),
        Rectangle{width, height} => *width * *height
    }
}

//  match over a mutable reference
fun scale_radius(self: &mut Rectangle, factor:  u64) {
    match (self) {
        Circle{radius: r} => *r = *r * factor,
        _                 => {} // do nothing if not a Circle
  }
}

// Patterns can be nested and contain conditions
fun nested_exp() {
let r : Result<Result<u64>> = Ok(Err(42));
let v = match (r) {
  Ok(Err(c)) if c <= 42   => 0,
  Ok(Err(c)) if c > 42   => 1,
  Ok(_) => 2,
  _ => 3
};
assert!(v == 1);
}

// Testing Enum Variants
fun test_enum_var() {
    let data: VersionedData;
if (data is VersionedData::V1) { };
assert!(data is V1|V2);
}

// Selecting From Enum Values
fun select_from_enum() {
    let s: String;
let data1 = VersionedData::V1{name: s};
let data2 = VersionedData::V2{name: s, age: 20};
assert!(data1.name == data2.name);
assert!(data2.age == 20); 
}

// Using Enums Patterns in Lets
fun use_enum_in_let() {
    let data: VersionData;
let V1{name} = data;
}

}
