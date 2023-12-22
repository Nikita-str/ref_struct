# About
Rust `proc_macro_attribute` that generate struct with ref fields.
# General Form & Args
general form is `#[ref_struct(arg[0], .., arg[n])]` where instead of each `arg[i]` can stay:
* `name(OutputStructName)`:
  <br/> by default output struct name is `Ref{input_struct_name}`;
  <br/> after this arg otput struct name will be changed to `OutputStructName`
* `clone(clone_field_1, .., clone_field_k)`:
  <br/> by default all fileds in output struct are refs;
  <br/> this arg specifies(by filed names) which fields must be cloned
* `ignore(ignore_field_1, .., ignore_field_k)`:
  <br/> by default all fileds are in output struct;
  <br/> this arg specifies(by field names) which fields must be ignored, they will not be in the output struct
* `derive(derive::path::one, .., derive::path::k)`:
  <br/> by default output struct have no derived traits;
  <br/> this arg specifies which traits must be derived
* `public`: make output struct `pub`
* `use_cow`: in output struct will be used `Cow<'_, Ty>` instead of `&'_ Ty`
* `ignore_struct(name(ignorestructName), derive(ignore::derive::path::one, .., ignore::derive::path::k))`:
  <br/> **!** imply `use_cow`;
  <br/> additionaly generate ignore struct: struct that contains ignored fields.
  <br/> additionaly generate `pub fn merge(self, ignored: IgnoredStructTy) -> InputStructTy` for output struct: allow to merge ref struct with ignored struct into initial struct.
# Examples
### simple example
```rust
#[ref_struct::ref_struct(
    ignore(lets_ignore_a, lets_ignore_b),
    derive(Clone, Debug, PartialEq, Eq),
    name(TwoVecAndClone),
    clone(lets_clone_a),
)]
struct InputStruct {
    lets_ignore_a: u32,
    lets_ref_a: Vec<u8>,
    lets_ref_b: Vec<u32>,
    lets_clone_a: u64,
    lets_ignore_b: Vec<Vec<u16>>,
}
```
will generate something like:
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
struct TwoVecAndClone<'x> {
    lets_ref_a: &'x Vec<u8>,
    lets_ref_b: &'x Vec<u32>,
    lets_clone_a: u64,
}
impl<'x> TwoVecAndClone<'x> {
    pub fn new(&InputStruct) -> Self { .. }
}
```
### use case
more real example you can find [here](./tests/serde_example.rs) and real use case [here](https://github.com/Nikita-str/rch/commit/3b84517af2d1131cd5e733fc0e5d5dd8eca35a61#diff-8f2478654b4cc05a52c3e85978b79cd1961c9856f217723c9aeac01365c7b1f3R189)

