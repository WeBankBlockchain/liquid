#![cfg_attr(not(feature = "std"), no_std)]

use liquid::storage;
use liquid_lang as liquid;
use liquid_lang::InOut;
use liquid_prelude::{string::String, vec::Vec};

#[derive(InOut)]
pub struct KVField {
    key: String,
    value: String,
}
#[derive(InOut)]
pub struct Entry {
    fileds: Vec<KVField>,
}

#[derive(InOut)]
pub enum Comparator {
    EQ(u8),
    NE(u8),
    GT(u8),
    GE(u8),
    LT(u8),
    LE(u8),
}

#[derive(InOut)]
pub struct CompareTriple {
    lvalue: String,
    rvalue: String,
    cmp: Comparator,
}

#[derive(InOut)]
pub struct Condition {
    cond_fields: Vec<CompareTriple>,
}

#[liquid::interface(name = auto)]
mod table {
    use super::*;

    extern "liquid" {
        fn createTable(
            &mut self,
            table_name: String,
            key: String,
            value_fields: String,
        ) -> i256;
        fn select(&self, table_name: String, condition: Condition) -> Vec<Entry>;
        fn insert(&mut self, table_name: String, entry: Entry) -> i256;
        fn update(
            &mut self,
            table_name: String,
            entry: Entry,
            condition: Condition,
        ) -> i256;
        fn remove(&mut self, table_name: String, condition: Condition) -> i256;
        fn desc(&self, table_name: String) -> (String, String);
    }
}

#[liquid::contract]
mod table_test {
    use super::{table::*, *};

    #[liquid(event)]
    struct InsertResult {
        count: i256,
    }

    #[liquid(event)]
    struct UpdateResult {
        count: i256,
    }
    #[liquid(event)]
    struct RemoveResult {
        count: i256,
    }

    #[liquid(storage)]
    struct TableTest {
        table: storage::Value<Table>,
    }

    #[liquid(methods)]
    impl TableTest {
        pub fn new(&mut self) {
            self.table
                .initialize(Table::at("/sys/table_storage".parse().unwrap()));
            self.table.createTable(
                String::from("t_test").clone(),
                String::from("id").clone(),
                [String::from("name").clone(), String::from("age").clone()].join(","),
            );
        }

        pub fn select(&mut self, id: String) -> (String, String) {
            let cmp_triple = CompareTriple {
                lvalue: String::from("id"),
                rvalue: id,
                cmp: Comparator::EQ(0),
            };
            let mut compare_fields = Vec::new();
            compare_fields.push(cmp_triple);
            let cond = Condition {
                cond_fields: compare_fields,
            };

            let entries = self.table.select(String::from("t_test"), cond).unwrap();

            if entries.len() < 1 {
                return (Default::default(), Default::default());
            }

            return (
                entries[0].fileds[0].value.clone(),
                entries[0].fileds[1].value.clone(),
            );
        }

        pub fn insert(&mut self, id: String, name: String, age: String) -> i256 {
            let kv0 = KVField {
                key: String::from("id"),
                value: id,
            };
            let kv1 = KVField {
                key: String::from("name"),
                value: name,
            };
            let kv2 = KVField {
                key: String::from("age"),
                value: age,
            };
            let mut kv_fields = Vec::new();
            kv_fields.push(kv0);
            kv_fields.push(kv1);
            kv_fields.push(kv2);
            let entry = Entry { fileds: kv_fields };
            let result = self.table.insert(String::from("t_test"), entry).unwrap();
            self.env().emit(InsertResult {
                count: result.clone(),
            });
            return result;
        }

        pub fn update(&mut self, id: String, name: String, age: String) -> i256 {
            let kv1 = KVField {
                key: String::from("name"),
                value: name,
            };
            let kv2 = KVField {
                key: String::from("age"),
                value: age,
            };
            let mut kv_fields = Vec::new();
            kv_fields.push(kv1);
            kv_fields.push(kv2);
            let entry = Entry { fileds: kv_fields };

            let cmp_triple = CompareTriple {
                lvalue: String::from("id"),
                rvalue: id,
                cmp: Comparator::EQ(0),
            };
            let mut compare_fields = Vec::new();
            compare_fields.push(cmp_triple);
            let cond = Condition {
                cond_fields: compare_fields,
            };

            let result = self
                .table
                .update(String::from("t_test"), entry, cond)
                .unwrap();
            self.env().emit(UpdateResult {
                count: result.clone(),
            });
            return result;
        }

        pub fn remove(&mut self, id: String) -> i256 {
            let cmp_triple = CompareTriple {
                lvalue: String::from("id"),
                rvalue: id,
                cmp: Comparator::EQ(0),
            };
            let mut compare_fields = Vec::new();
            compare_fields.push(cmp_triple);
            let cond = Condition {
                cond_fields: compare_fields,
            };
            let result = self.table.remove(String::from("t_test"), cond).unwrap();
            self.env().emit(RemoveResult {
                count: result.clone(),
            });
            return result;
        }
    }
}
