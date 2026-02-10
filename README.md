# Concept_db 

A DB designed to take in your struct and store across an array of JSON files. 

## Basic Usage 

the following examples will follow the example of basic usage with Dog. 

```rust
use concept_db::elaborate::Fragment; 

struct Dog {
    name: String, 
    age: i8
}

// impl Dog....

fn main() {
    let default_fragment: Fragment<Dog> = Fragment::new(Dog::default()); 
    default_fragment.create_table("dog_table".to_string()); 
}
```

## update tables 

```rust
fn main() {
    let default_fragment: Fragment<Dog> = Fragment::new(Dog::default());
    // updates specific key, value pair 
    default_fragment.update_table("dog_table".to_string(), "name".to_string(), "default_name".to_string()); 
}
```

## delete tables 

```rust
fn main() {
    let default_fragment: Fragment<Dog> = Fragment::new(Dog::default());
    default_fragment.delete("dog_table".to_string())
} 
```

### The above example are only a part of the functionality.

There is still much to do and I look forward to continue working on this project. 

## Anticipated updates

    - Improved error handling. 
    - simplification in some areas. 
    - Functions will only alter tables corresponding to T. 