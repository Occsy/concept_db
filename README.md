# Concept_db 

A DB designed to take in your struct and store across an array of JSON files. 

## Basic Usage 

the following examples will follow the example of basic usage with Dog. 

```rust
use concept_db::elaborate::Fragment;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Dog {
    name: String,
    age: i8,
}

impl Default for Dog {
    fn default() -> Self where Self: DeserializeOwned{
        Self {
            name: "dog_name".to_string(),
            age: 3,
        }
    }
}

impl Dog {
    fn create_dog_table() -> std::io::Result<()> {
        let dog_fragment: Fragment<Dog> = Fragment::new(Dog::default());

        dog_fragment.create_table("dog_table".to_string());

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    Dog::create_dog_table()?;
    Ok(())
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
    default_fragment.delete("dog_table".to_string()); 
} 
```

### The above example are only a part of the functionality.

There is still much to do and I look forward to continue working on this project. 

### Also...

I would love any recommendations and help if able. 


## Anticipated updates

    - Logger improvements (added in version: 0.1.230) 
    - Collection improvements
    - Functions will only alter tables corresponding to T. 