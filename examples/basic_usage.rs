use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use derive_overwrites::*;

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

#[derive(Clone, Debug)]
struct MyStruct {
    pub count: usize,
}

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
//

// By specifying the type parameter, we can create functions of the same
// name as the original, which works on the MyWrapper version instead of
// the inner type
impl MyStructOverwrites for MyWrapper<MyStruct> {
    fn increment_by(&mut self, amount: usize) {
        println!("OVERWRITTEN: Incrementing by amount {amount}");
        self.inner.increment_by(amount);
        println!("Additional data: {}", self.additional_data);
    }
}

fn main() {
    // Structs can be converted to WithId<_> through `From`/`Into`
    let mut my_struct = MyWrapper {
        additional_data: String::from("Hello, World!"),
        inner: MyStruct { count: 0 },
    };

    println!("Initial Value:");
    println!("{my_struct:#?}");
    println!();

    // WithId automatically dereferences to its inner value
    my_struct.count += 1;
    println!("Updating the value directly through DerefMut:");
    println!("{my_struct:#?}");
    println!();

    // This also works for function calls
    my_struct.increment();
    println!("Updating by calling a function of the inner type:");
    println!("{my_struct:#?}");
    println!();

    // This function was overwritten by creating an impl for `WithId<MyStruct>`
    // and implementing a function with the same name as the original
    my_struct.increment_by(3);
    println!("Updating by calling the 'overwritten' function:");
    println!("{my_struct:#?}");
    println!();

    // The original function can still be accessed by manually dereferencing
    (*my_struct).increment_by(5);
    println!("Updating by calling the 'original' function, by dereferencing first:");
    println!("{my_struct:#?}");
    println!();
}
