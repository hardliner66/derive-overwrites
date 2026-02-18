# derive-overwrites

A Rust attribute macro to create a trait from an impl block, to allow overwriting functions in a way that is compatible with using an external wrapper struct.

## Usage

Imagine you are using a wrapper struct to add additional data to a type.
The wrapper implements deref and deref_mut, so you can access the inner type's methods and fields transparently.

Something like this:

```rs
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

struct MyWrapper<T> {
    pub additional_data: String,
    pub inner: T,
}

impl<T: Debug> Debug for MyWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyWrapper")
            .field("additional_data", &self.additional_data)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Deref for MyWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for MyWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
```

Now you want make use of the additional data while calling a function of the inner type, but you don't want to change the function signature.

If the wrapper is defined in the same crate is the inner type, you can just create an impl for the wrapper and call into the function of the inner type
like this:

```rs
#[derive(Clone, Debug)]
struct MyStruct {
    pub count: usize,
}

impl MyStruct {
    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn increment_by(&mut self, amount: usize) {
        self.count += amount;
    }
}

impl MyWrapper<MyStruct> {
    fn increment_by(&mut self, amount: usize) {
        println!("OVERWRITTEN: Incrementing by amount {amount}");
        self.inner.increment_by(amount);
        println!("Additional data: {}", self.additional_data);
    }
}
```

The problem is, this wont work if the wrapper is defined in a different crate than the inner type.
But there is another trick we can use. By defining a trait that contains the functions we want to overwrite,
we can implement the trait for the wrapper and overwrite functions this way.

```rs
trait MyStructOverwrites {
    fn increment_by(&mut self, amount: usize);
}

impl MyStructOverwrites for MyWrapper<MyStruct> {
    fn increment_by(&mut self, amount: usize) {
        println!("OVERWRITTEN: Incrementing by amount {amount}");
        self.inner.increment_by(amount);
        println!("Additional data: {}", self.additional_data);
    }
}
```

This works fine, but now you have to manually keep the trait and its implementation in up to date.

That's where derive_overwrites comes into play. It uses an attribute macro to generate the trait for you,
simplifying the process of keeping the trait and the original type impl in sync:

```rs
use derive_overwrites::*;

// Generate overwrites for MyStruct
#[generate_overwrites]
impl MyStruct {
    // Mark functions that should not be overwritten with #[skip]
    #[skip]
    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn increment_by(&mut self, amount: usize) {
        self.count += amount;
    }
}

// // Alternatively, use #[generate_overwrites(all = false)] to disable all overwrites by default
// #[generate_overwrites(all = false)]
// impl MyStruct {
//     pub fn increment(&mut self) {
//         self.count += 1;
//     }

//     // and explicitly include the functions you want with #[overwrite]
//     #[overwrite]
//     pub fn increment_by(&mut self, amount: usize) {
//         self.count += amount;
//     }
// }
```
