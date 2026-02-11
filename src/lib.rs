pub mod elaborate {
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::to_string;
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::fs::{self, DirEntry, File};
    use std::io::{BufReader, Read, Write};
    use std::path::Path;

    #[derive(Debug)]
    /// All errors converted into types from this enum 
    pub enum TErrors {
        /// file doesnt exist
        FileNotFound,
        /// error related file functions outside read/write
        FileError,
        /// error manipulating Dir 
        DirError,
        /// error reading file 
        ReadByteError,
        /// error writing directory 
        WriteByteError,
        /// error with conversions involving a string 
        StringConvert,
        /// error converting hash
        HashConvert,
        /// error deleting file 
        DeleteError,
    }

    /// covers the different kinds of HashMap that may be required for conversion
    /// from intended struct.
    pub trait ToHash {
        fn to_hash(&self) -> Result<HashMap<String, String>, TErrors>;
        fn to_hash_opt(&self) -> Result<HashMap<String, Option<String>>, TErrors>;
        fn to_hash_vec(&self) -> Result<HashMap<String, Vec<String>>, TErrors>;
    }

    /// simplifies code in ToHash trait 
    // why did I even make this??
    trait ToHashOpt {
        fn convert_opt(&self) -> HashMap<String, Option<String>>;
    }

    impl ToHashOpt for HashMap<String, String> {
        fn convert_opt(&self) -> HashMap<String, Option<String>> {
            let mut temp_hash: HashMap<String, Option<String>> = HashMap::new();

            self.into_iter().for_each(|(k, v)| {
                let v: Option<String> = if v.len() > 0 { Some(v.clone()) } else { None };

                temp_hash.insert(k.clone(), v);
            });

            temp_hash
        }
    }

    /// added functionality to count iterations of an item in a vector
    /// of a generic type
    pub fn count_val<T: std::fmt::Debug + Eq>(vec: Vec<T>, value: String) -> usize {
        vec.into_iter()
            .map(|c| format!("{c:?}").trim().to_string())
            .filter(|v| *v == value.trim().to_string())
            .collect::<Vec<String>>()
            .len()
    }

    /// converts HashMap<String, String> to T
    pub fn read_hash<T: Serialize + DeserializeOwned + Sized + Clone>(
        hash: HashMap<String, String>,
    ) -> Result<T, TErrors> {
        let Ok(convert1) = serde_json::to_string(&hash) else {
            return Err(TErrors::StringConvert);
        };

        let Ok(convert2) = serde_json::from_str(&convert1) else {
            return Err(TErrors::StringConvert);
        };

        Ok(convert2)
    }

    #[derive(Debug, Serialize)]
    /// Takes in a struct of type T,
    /// then applies concept_db functionality
    pub struct Fragment<T: Serialize + DeserializeOwned + Sized + Clone> {
        pub inner: T,
    }

    impl<T: Serialize + DeserializeOwned + Sized + Clone + Debug> ToString for Fragment<T> {
        /// converts T to String
        fn to_string(&self) -> String {
            let Ok(output) = to_string(&self.inner) else {
                panic!("unable to convert: {self:?}");
            };

            output
        }
    }

    impl<T: Serialize + DeserializeOwned + Sized + Clone + Debug> ToHash for Fragment<T> {
        /// converts T to HashMap<String, String>
        fn to_hash(&self) -> Result<HashMap<String, String>, TErrors>
        where
            HashMap<String, String>: Serialize,
        {
            let Ok(output) = serde_json::from_str::<HashMap<String, String>>(&self.to_string())
            else {
                return Err(TErrors::StringConvert);
            };

            Ok(output)
        }
        /// converts T to HashMap<String, Option<String>>
        fn to_hash_opt(&self) -> Result<HashMap<String, Option<String>>, TErrors>
        where
            HashMap<String, Option<String>>: Serialize,
        {
            let Ok(output) = self.to_hash() else {
                return Err(TErrors::HashConvert);
            };

            Ok(output.convert_opt())
        }

        /// converts T to HashMap<String, Vec<String>>
        fn to_hash_vec(&self) -> Result<HashMap<String, Vec<String>>, TErrors> {
            let Ok(output) =
                serde_json::from_str::<HashMap<String, Vec<String>>>(&self.to_string())
            else {
                return Err(TErrors::StringConvert);
            };

            Ok(output)
        }
    }

    impl<T: Serialize + DeserializeOwned + Sized + Clone + Debug> Fragment<T> {
        /// initializes the Fragment<T>
        pub fn new(inner: T) -> Self
        where
            T: Serialize + DeserializeOwned + Sized + Clone,
        {
            Self { inner }
        }

        /// parses file if exists then returns a new Fragment<T>
        pub fn read_table(&self, file_path: String) -> Result<Fragment<T>, TErrors> {
            let path: String = format!("./db_files/{}.json", file_path);

            let convert_path: &Path = Path::new(&path);

            if !convert_path.exists() {
                return Err(TErrors::FileNotFound);
            }

            let Ok(f) = File::open(convert_path) else {
                return Err(TErrors::FileError);
            };

            let reader: BufReader<File> = BufReader::new(f);

            let Ok(inner_value) = serde_json::from_reader(reader) else {
                return Err(TErrors::ReadByteError);
            };

            let new_value: Fragment<T> = Self::new(inner_value);

            Ok(new_value)
        }

        /// creates a new json file and inputs the table
        /// created using the struct
        pub fn create_table(&self, table_name: String) -> Result<&Self, TErrors> {
            let path_root: &String = &String::from("./db_files/");

            if !Path::new(path_root).is_dir() {
                std::fs::create_dir(path_root).map_err(|_| {
                    return TErrors::DirError;
                })?;
            }

            if Path::new(&format!("{}{}.json", path_root, table_name)).is_file() {
                return Ok(self);
            }

            let string_convert: String = self.to_string();

            let mut src_bytes = string_convert.as_bytes();

            let inner_path: &String = &format!("{path_root}{}", table_name + ".json").to_string();

            let full_path: &Path = &Path::new(inner_path);

            let Ok(mut file) = File::create_new(full_path) else {
                return Err(TErrors::FileError);
            };

            file.write(&mut src_bytes).map_err(|_| {
                return TErrors::WriteByteError;
            })?;

            Ok(self)
        }

        /// deletes the table in question
        pub fn delete_table(&self, table_name: String) -> std::io::Result<()> {
            let path: &String = &format!("./db_files/{}.json", table_name);

            let convert_path: &Path = Path::new(path);

            if convert_path.is_file() {
                std::fs::remove_file(path)?
            }

            Ok(())
        }
        /// deletes table if tables contents match T of Fragment<T>
        pub fn delete_table_infer(&self) -> Result<(), TErrors>
        where
            Fragment<T>: DeserializeOwned,
        {
            let Ok(dir) = fs::read_dir("./db_files/") else {
                return Err(TErrors::DirError);
            };
            for entry in dir.into_iter().filter_map(|f| f.ok()) {
                let mut context = [0; 10];
                let Ok(mut file) = File::open(entry.path()) else {
                    return Err(TErrors::FileError);
                };
                file.read(&mut context).map_err(|_| {
                    return TErrors::ReadByteError;
                })?;
                let string_context: String = String::from_utf8_lossy(&context).to_string();
                if let Ok(_) = serde_json::from_str::<Self>(&string_context) {
                    fs::remove_file(entry.path()).map_err(|_| {
                        return TErrors::DeleteError;
                    })?;
                } else {
                    continue;
                }
            }
            Ok(())
        }

        /// returns all regardless of type of T
        pub fn get_all(&self) -> Result<Vec<HashMap<String, String>>, TErrors> {
            let mut temp_vec: Vec<HashMap<String, String>> = Vec::new();
            for entry in fs::read_dir("./db_files/")
                .map_err(|_| {
                    return TErrors::DirError;
                })?
                .into_iter()
                .filter_map(|f| f.ok())
                .collect::<Vec<DirEntry>>()
            {
                let Ok(obj) = self.read_table(entry.path().to_string_lossy().to_string()) else {
                    return Err(TErrors::ReadByteError);
                };

                let Ok(obj_hash) = obj.to_hash() else {
                    return Err(TErrors::HashConvert);
                };
                temp_vec.push(obj_hash);
            }
            Ok(temp_vec)
        }

        /// returns only matching initial type of T
        pub fn get_all_infer(&self) -> std::io::Result<Vec<Fragment<T>>> {
            let mut temp_vec: Vec<Fragment<T>> = Vec::new();

            for entry in fs::read_dir("./db_files/")?
                .into_iter()
                .filter_map(|f| f.ok())
            {
                if let Ok(obj) = self.read_table(entry.path().to_string_lossy().to_string()) {
                    temp_vec.push(obj);
                } else {
                    continue;
                }
            }

            Ok(temp_vec)
        }

        /// combines 2 tables
        pub fn merge(
            &self,
            foreign_table: HashMap<String, String>,
        ) -> Result<HashMap<String, String>, TErrors> {
            let Ok(mut hashed_self) = self.to_hash() else {
                return Err(TErrors::HashConvert);
            };
            for (key, value) in foreign_table {
                hashed_self.insert(key, value);
            }

            Ok(hashed_self)
        }

        /// performs a left join on 2 tables
        pub fn left_join(
            &self,
            foreign_table: HashMap<String, Option<String>>,
        ) -> Result<HashMap<String, String>, serde_json::Error> {
            let mut self_hashed: HashMap<String, String> = self.to_hash().unwrap();

            let key_vals: Vec<(String, String)> = foreign_table
                .iter()
                .filter(|(k, _)| self_hashed.contains_key(*k))
                .map(|(k2, v2)| {
                    let v2: &Option<String> = if v2.is_some() {
                        v2
                    } else {
                        &Some("None".to_string())
                    };
                    return (
                        format!(
                            "{}_{}",
                            k2.clone(),
                            count_val(foreign_table.iter().collect(), k2.clone())
                        ),
                        v2.clone().unwrap(),
                    );
                })
                .collect();

            key_vals.into_iter().for_each(|(k, v)| {
                self_hashed.insert(k, v);
            });

            Ok(self_hashed)
        }

        /// sorts tables based on key + value
        pub fn build_where(&self, key: String, value: String) -> Vec<Self> {
            let mut temp_vec: Vec<Fragment<T>> = Vec::new();
            for entry in fs::read_dir("./db_files/")
                .unwrap()
                .into_iter()
                .filter_map(|f| f.ok())
            {
                let contents: Fragment<T> = self
                    .read_table(entry.path().to_string_lossy().to_string())
                    .unwrap();
                let hashed_contents: HashMap<String, String> = contents.to_hash().unwrap();
                if hashed_contents.contains_key(&key)
                    && hashed_contents.get(&key).unwrap().trim().to_string()
                        == value.trim().to_string()
                {
                    temp_vec.push(contents)
                }
            }
            temp_vec
        }

        /// update key in table
        pub fn update_table(&self, table_name: String, key: String, value: String) -> T {
            let current_table: Self = self.read_table(table_name).unwrap();

            let mut hashed_table: HashMap<String, String> = current_table.to_hash().unwrap();

            hashed_table.insert(key, value);

            serde_json::from_str::<T>(&serde_json::to_string(&hashed_table).unwrap()).unwrap()
        }

        /// updates table's key and value if value is type of Vec<String>
        pub fn update_table_vec(&self, table_name: String, key: String, value: Vec<String>) -> T {
            let current_table: Self = self.read_table(table_name).unwrap();

            let mut hashed_table: HashMap<String, Vec<String>> =
                current_table.to_hash_vec().unwrap();

            hashed_table.insert(key, value);

            serde_json::from_str::<T>(&serde_json::to_string(&hashed_table).unwrap()).unwrap()
        }
    }
}
