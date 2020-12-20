#![cfg_attr(not(feature = "std"), no_std)]
#![feature(unboxed_closures, fn_traits)]

use lazy_static::lazy_static;
use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod entry {
    extern "liquid" {
        fn getInt(&self, key: String) -> i256;
        fn getUint(&self, key: String) -> u256;
        fn getAddress(&self, key: String) -> address;
        fn getString(&self, key: String) -> String;

        fn set(&mut self, key: String, value: i256);
        fn set(&mut self, key: String, value: u256);
        fn set(&mut self, key: String, value: address);
        fn set(&mut self, key: String, value: String);
    }
}

#[liquid::interface(name = auto)]
mod kv_table {
    use super::entry::*;

    extern "liquid" {
        fn get(&self, primary_key: String) -> (bool, Entry);
        #[liquid(mock_context_getter = "liquid_is_fun")]
        fn set(&mut self, primary_key: String, entry: Entry) -> i256;
        fn newEntry(&self) -> Entry;
    }
}

#[liquid::interface(name = auto)]
mod kv_table_factory {
    use super::kv_table::*;

    extern "liquid" {
        fn openTable(&self, name: String) -> KvTable;
        fn createTable(
            &mut self,
            name: String,
            primary_key: String,
            fields: String,
        ) -> i256;
    }
}

#[liquid::contract]
mod kv_table_test {
    use super::{kv_table_factory::*, *};
    use liquid_core::storage;

    #[liquid(storage)]
    struct KvTableTest {
        table_factory: storage::Value<KvTableFactory>,
    }

    #[liquid(event)]
    struct SetResult {
        count: i256,
    }

    lazy_static! {
        static ref TABLE_NAME: String = String::from("t_kvtest");
        static ref ID_FIELD: String = String::from("id");
        static ref PRICE_FIELD: String = String::from("item_field");
        static ref NAME_FIELD: String = String::from("item_name");
    }

    #[liquid(methods)]
    impl KvTableTest {
        pub fn new(&mut self) {
            self.table_factory
                .initialize(KvTableFactory::at("0x1010".parse().unwrap()));
            self.table_factory.createTable(
                TABLE_NAME.clone(),
                ID_FIELD.clone(),
                [PRICE_FIELD.clone(), NAME_FIELD.clone()].join(","),
            );
        }

        pub fn get(&self, id: String) -> (bool, i256, String) {
            let table = self.table_factory.openTable(TABLE_NAME.clone()).unwrap();
            if let Some((ok, entry)) = table.get(id) {
                return (
                    ok,
                    entry.getInt(String::from("item_price")).unwrap(),
                    entry.getString(String::from("item_name")).unwrap(),
                );
            }
            return (false, 0.into(), Default::default());
        }

        pub fn set(&mut self, id: String, item_price: i256, item_name: String) -> i256 {
            let table = self.table_factory.openTable(TABLE_NAME.clone()).unwrap();
            let entry = table.newEntry().unwrap();
            (entry.set)(String::from("id"), id.clone());
            (entry.set)(String::from("item_price"), item_price);
            (entry.set)(String::from("item_name"), item_name);
            let count = table.set(id, entry).unwrap();

            self.env().emit(SetResult {
                count: count.clone(),
            });
            count
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{entry::*, kv_table::*};
        use predicates::prelude::*;

        #[test]
        fn get_works() {
            // EXPECTATIONS SETUP
            let create_table_ctx = KvTableFactory::createTable_context();
            create_table_ctx.expect().returns(0);

            let open_table_ctx = KvTableFactory::openTable_context();
            open_table_ctx
                .expect()
                .returns(KvTable::at(Default::default()));

            let get_ctx = KvTable::get_context();
            get_ctx
                .expect()
                .when(predicate::eq(String::from("cat")))
                .returns((true, Entry::at(Default::default())));
            get_ctx
                .expect()
                .when_fn(|primary_key| primary_key == "dog")
                .throws();

            let get_int_ctx = Entry::getInt_context();
            get_int_ctx.expect().returns(2500);

            let get_string_ctx = Entry::getString_context();
            get_string_ctx.expect().returns("dounai");

            // TESTS BEGIN
            let contract = KvTableTest::new();

            let (success, price, name) = contract.get(String::from("cat"));
            assert_eq!(success, true);
            assert_eq!(price, 2500.into());
            assert_eq!(name, "dounai");

            let (success, price, name) = contract.get(String::from("dog"));
            assert_eq!(success, false);
            assert_eq!(price, 0.into());
            assert_eq!(name, "");
        }

        #[test]
        fn set_works() {
            use std::collections::HashMap;

            static mut ENTRY: HashMap<String, Vec<u8>> = HashMap::new();

            // EXPECTATIONS SETUP
            let create_table_ctx = KvTableFactory::createTable_context();
            create_table_ctx.expect().returns(0);

            let open_table_ctx = KvTableFactory::openTable_context();
            open_table_ctx
                .expect()
                .returns(KvTable::at(Default::default()));

            let new_entry_ctx = KvTable::newEntry_context();
            new_entry_ctx
                .expect()
                .returns(Entry::at(Default::default()));

            let entry_set_ctx = Entry::set_context();
            entry_set_ctx
                .expect::<(String, String)>()
                .returns_fn(|key, value| {
                    ENTRY.insert(key, value.into_bytes());
                });
            entry_set_ctx
                .expect::<(String, i256)>()
                .returns_fn(|key, value| {
                    ENTRY.insert(key, value.to_be_bytes().to_vec());
                });

            let get_ctx = KvTable::get_context();
            get_ctx
                .expect()
                .when(predicate::eq(String::from("dog")))
                .returns((true, Entry::at(Default::default())));

            let get_int_ctx = Entry::getInt_context();
            get_int_ctx
                .expect()
                .returns_fn(|key| i256::from_signed_be_bytes(ENTRY.get(&key).unwrap()));

            let get_string_ctx = Entry::getString_context();
            get_string_ctx.expect().returns_fn(|key| {
                String::from_utf8(ENTRY.get(&key).unwrap().clone()).unwrap()
            });

            let kv_table_set_ctx = KvTable::liquid_is_fun();
            kv_table_set_ctx.expect().returns(0);

            // TESTS BEGIN
            let mut contract = KvTableTest::new();

            contract.set(String::from("dog"), 2000.into(), String::from("baicai"));
            let (success, price, name) = contract.get(String::from("dog"));
            assert_eq!(success, true);
            assert_eq!(price, 2000.into());
            assert_eq!(name, "baicai");
        }
    }
}
