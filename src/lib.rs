pub mod elaborate {
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::to_string;
    use std::{
        collections::HashMap,
        fmt::Debug,
        fs::{self, DirEntry, File},
        io::{BufReader, Read, Write},
        path::Path,
    };

    #[derive(Default, Debug, Clone)]
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
        /// error reading collection
        CollectReadError,
        /// not ideal but meant for ease of use with Commit
        #[default]
        None,
    }

    /// An output design for the Logger
    pub struct Commit<T: Serialize + DeserializeOwned + Sized + Clone + Debug> {
        pub success: bool,
        pub package: Result<T, TErrors>,
        pub collection: Result<Collection<T>, TErrors>,
    }

    impl<T: Serialize + DeserializeOwned + Sized + Clone + Debug> Default for Commit<T> {
        fn default() -> Self {
            Self {
                success: false,
                package: Err(TErrors::default()),
                collection: Err(TErrors::default()),
            }
        }
    }

    impl<T: Serialize + DeserializeOwned + Sized + Clone + Debug> Commit<T> {
        pub fn determine(
            &self,
            package: Result<T, TErrors>,
            collection: Result<Collection<T>, TErrors>,
        ) -> Self {
            let success: bool = if package.is_err() && collection.is_err() {
                false
            } else {
                true
            };

            Self {
                success,
                package,
                collection,
            }
        }
    }

    pub trait Collect<T: Serialize + DeserializeOwned + Sized + Clone + Debug> {
        /// collects all tables across the JSON files that match type of T.
        fn collect(&self, frag: Fragment<T>) -> Result<Self, TErrors>
        where
            Self: Sized;
        /// adds a table of type T to collection.
        fn append(&mut self, obj: T) -> Self;
        /// removes an object of type T from the collection.
        fn remove(&self, obj: T) -> Self
        where
            T: PartialEq;
        /// update an index to the provided object of type T.
        fn update_index(&self, index: usize, new_obj: T) -> Self;
        /// write collection to json file.  
        fn write_to_file(&self, title: String) -> Result<(), TErrors>
        where
            Self: Serialize + DeserializeOwned + Clone + Debug;
    }

    /// covers the different kinds of HashMap that may be required for conversion
    /// from intended struct.
    pub trait ToHash {
        /// converts T to HashMap.
        fn to_hash(&self) -> Result<HashMap<String, String>, TErrors>;
        /// converts T to HashMap of String and Option of String.
        fn to_hash_opt(&self) -> Result<HashMap<String, Option<String>>, TErrors>;
        /// converts T to HashMap of String and Vec of String.
        fn to_hash_vec(&self) -> Result<HashMap<String, Vec<String>>, TErrors>;
        /// converts hashmap to Vec of tuple of String and String.
        fn zip(&self) -> Result<Vec<(String, String)>, TErrors>;
    }

    pub trait ToLog<T: Serialize + DeserializeOwned + Sized + Clone + Debug> {
        /// updates initial state
        fn set_prior(&self, prior: T) -> Self;
        /// intended to set updated state on completion for comparision (to be developed)
        fn set_later(&self, later: T) -> Self;
        /// sets the time at which change occured.
        fn set_time_stamp(&self, time_stamp: String) -> Self;
        /// this is experimental. it wont work for HashMap of String and Vec of T
        fn raw_changes(&self) -> Result<(Vec<(String, String)>, Vec<(String, String)>), TErrors>;
        /// measures success of execution
        fn commit(&self) -> Commit<T>;
    }

    pub trait ToLogCollect<T: Serialize + DeserializeOwned + Sized + Clone + Debug> {
        /// updates initial state
        fn set_prior(&self, prior: Collection<T>) -> Self;
        /// intended to set updated state on completion for comparision (to be developed)
        fn set_later(&self, later: Collection<T>) -> Self;
        /// sets the time at which change occured.
        fn set_time_stamp(&self, time_stamp: String) -> Self;
        /// makes comparison between before and after (still under development)
        /// This has a lot more to be added
        fn raw_changes_collect(&self) -> Result<(Vec<T>, Vec<T>), TErrors>;
        /// measures success of execution
        fn commit(&self) -> Commit<T>;
    }

    /// simplifies code in ToHash trait
    // why did I even make this??
    trait ToHashOpt {
        /// converts T to HashMap of String and Option of String.
        fn convert_opt(&self) -> HashMap<String, Option<String>>;
    }

    impl ToHashOpt for HashMap<String, String> {
        /// impl of ToHashOpt.convert_opt()
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

    /// converts HashMap of String and String to T
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

        fn zip(&self) -> Result<Vec<(String, String)>, TErrors> {
            let mut _temp_vec: Vec<(String, String)> = Vec::new();
            if let Ok(vec_hash) = self.to_hash() {
                let keys = vec_hash.clone().into_keys().collect::<Vec<String>>();
                let values = vec_hash.into_values().collect::<Vec<String>>();
                _temp_vec = keys.iter().map(|d| d.clone()).zip(values).collect();
                return Ok(_temp_vec);
            } else {
                println!("This is an error still in work. Try raw_changes_collect.");
                return Err(TErrors::HashConvert);
            }
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
        pub fn get_all_infer(&self) -> Result<Vec<Fragment<T>>, TErrors> {
            let mut temp_vec: Vec<Fragment<T>> = Vec::new();

            for entry in fs::read_dir("./db_files/")
                .map_err(|_| {
                    return TErrors::DirError;
                })?
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
        ) -> Result<HashMap<String, String>, TErrors> {
            let Ok(mut self_hashed) = self.to_hash() else {
                return Err(TErrors::HashConvert);
            };

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
                        v2.clone().unwrap_or("".to_string()),
                    );
                })
                .collect();

            key_vals.into_iter().for_each(|(k, v)| {
                self_hashed.insert(k, v);
            });

            Ok(self_hashed)
        }

        /// sorts tables based on key + value
        pub fn build_where(&self, key: String, value: String) -> Result<Vec<Self>, TErrors> {
            let mut temp_vec: Vec<Fragment<T>> = Vec::new();
            for entry in fs::read_dir("./db_files/")
                .map_err(|_| {
                    return TErrors::DirError;
                })?
                .into_iter()
                .filter_map(|f| f.ok())
            {
                let Ok(contents) = self.read_table(entry.path().to_string_lossy().to_string())
                else {
                    return Err(TErrors::ReadByteError);
                };
                let Ok(hashed_contents) = contents.to_hash() else {
                    return Err(TErrors::HashConvert);
                };
                if hashed_contents.contains_key(&key)
                    && hashed_contents
                        .get(&key)
                        .unwrap_or(&"".to_string())
                        .trim()
                        .to_string()
                        == value.trim().to_string()
                {
                    temp_vec.push(contents)
                }
            }
            Ok(temp_vec)
        }

        /// update key in table
        pub fn update_table(
            &self,
            table_name: String,
            key: String,
            value: String,
        ) -> Result<T, TErrors> {
            let Ok(current_table) = self.read_table(table_name) else {
                return Err(TErrors::FileError);
            };

            let Ok(mut hashed_table) = current_table.to_hash() else {
                return Err(TErrors::HashConvert);
            };

            hashed_table.insert(key, value);

            let Ok(hash_to_string) = &serde_json::to_string(&hashed_table) else {
                return Err(TErrors::StringConvert);
            };

            let Ok(output) = serde_json::from_str::<T>(hash_to_string) else {
                return Err(TErrors::StringConvert);
            };

            Ok(output)
        }

        /// updates table's key and value if value is type of Vec<String>
        pub fn update_table_vec(
            &self,
            table_name: String,
            key: String,
            value: Vec<String>,
        ) -> Result<T, TErrors> {
            let Ok(current_table) = self.read_table(table_name) else {
                return Err(TErrors::FileError);
            };

            let Ok(mut hashed_table) = current_table.to_hash_vec() else {
                return Err(TErrors::HashConvert);
            };

            hashed_table.insert(key, value);

            let Ok(hash_to_string) = &serde_json::to_string(&hashed_table) else {
                return Err(TErrors::HashConvert);
            };

            let Ok(output) = serde_json::from_str::<T>(hash_to_string) else {
                return Err(TErrors::StringConvert);
            };

            Ok(output)
        }
    }

    #[derive(Default, Clone)]
    /// creates a collection of type T
    pub struct Collection<T: Serialize + DeserializeOwned + Sized + Clone> {
        pub inner: Vec<T>,
    }

    impl<T: Serialize + DeserializeOwned + Sized + Clone + Debug> Collect<T> for Collection<T> {
        fn collect(&self, frag: Fragment<T>) -> Result<Self, TErrors>
        where
            Self: Sized,
        {
            let all_inferred: Vec<T> = frag
                .get_all_infer()
                .unwrap()
                .into_iter()
                .map(|f| f.inner)
                .collect();

            Ok(Self {
                inner: all_inferred,
            })
        }

        fn append(&mut self, obj: T) -> Self {
            self.inner.push(obj);
            Self {
                inner: self.inner.clone(),
            }
        }

        fn remove(&self, obj: T) -> Self
        where
            T: PartialEq,
        {
            Self {
                inner: self
                    .inner
                    .clone()
                    .into_iter()
                    .filter(|f| *f != obj)
                    .collect::<Vec<T>>(),
            }
        }

        fn update_index(&self, index: usize, new_obj: T) -> Self {
            Self {
                inner: self
                    .inner
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut f)| {
                        if i == index {
                            f = new_obj.clone()
                        }
                        f
                    })
                    .collect::<Vec<T>>(),
            }
        }

        fn write_to_file(&self, title: String) -> Result<(), TErrors>
        where
            Self: Serialize + DeserializeOwned + Clone + Debug,
        {
            let frag: Fragment<Self> = Fragment::new(self.clone());

            frag.create_table(title).map_err(|_| {
                return TErrors::FileError;
            })?;

            Ok(())
        }
    }

    /// A simple logger for actions done
    pub struct Logger<T: Serialize + DeserializeOwned + Sized + Clone + Debug> {
        pub prior: T,
        pub later: Result<T, TErrors>,
        pub time_stamp: String,
    }

    impl<T: Serialize + DeserializeOwned + Sized + Clone + Debug> ToLog<T> for Logger<T> {
        fn set_prior(&self, prior: T) -> Self {
            Self {
                prior: prior.clone(),
                later: self.later.clone(),
                time_stamp: self.time_stamp.clone(),
            }
        }

        fn set_later(&self, later: T) -> Self {
            Self {
                prior: self.prior.clone(),
                later: Ok(later.clone()),
                time_stamp: self.time_stamp.clone(),
            }
        }

        fn set_time_stamp(&self, time_stamp: String) -> Self {
            Self {
                prior: self.prior.clone(),
                later: self.later.clone(),
                time_stamp: time_stamp.clone(),
            }
        }

        fn raw_changes(&self) -> Result<(Vec<(String, String)>, Vec<(String, String)>), TErrors> {
            let prior_frag: Fragment<T> = Fragment::new(self.prior.clone());
            let later_frag: Fragment<T> = Fragment::new(self.later.clone()?);
            let left_vec: Vec<(String, String)> = prior_frag.zip()?;
            let right_vec: Vec<(String, String)> = later_frag.zip()?;
            Ok((left_vec, right_vec))
        }

        fn commit(&self) -> Commit<T> {
            Commit::default().determine(self.later.clone(), Err(TErrors::None))
        }
    }
    /// A logger for Collection struct.
    pub struct CollectLogger<T: Serialize + DeserializeOwned + Sized + Clone + Debug> {
        pub prior: Collection<T>,
        pub later: Result<Collection<T>, TErrors>,
        pub time_stamp: String,
    }

    impl<T: Serialize + DeserializeOwned + Sized + Clone + Debug + PartialEq> ToLogCollect<T>
        for CollectLogger<T>
    {
        fn set_prior(&self, prior: Collection<T>) -> Self {
            Self {
                prior: prior.clone(),
                later: self.later.clone(),
                time_stamp: self.time_stamp.clone(),
            }
        }

        fn set_later(&self, later: Collection<T>) -> Self {
            Self {
                prior: self.prior.clone(),
                later: Ok(later.clone()),
                time_stamp: self.time_stamp.clone(),
            }
        }

        fn set_time_stamp(&self, time_stamp: String) -> Self {
            Self {
                prior: self.prior.clone(),
                later: self.later.clone(),
                time_stamp: time_stamp.clone(),
            }
        }

        fn raw_changes_collect(&self) -> Result<(Vec<T>, Vec<T>), TErrors> {
            let added: Vec<T> = self
                .later
                .clone()?
                .inner
                .clone()
                .into_iter()
                .filter(|f| !self.prior.inner.contains(f))
                .collect();
            let removed: Vec<T> = self
                .prior
                .inner
                .clone()
                .into_iter()
                .filter(|f| !self.later.clone().unwrap().inner.contains(f))
                .collect();
            Ok((added, removed))
        }

        fn commit(&self) -> Commit<T> {
            Commit::default().determine(Err(TErrors::None), self.later.clone())
        }
    }
}
