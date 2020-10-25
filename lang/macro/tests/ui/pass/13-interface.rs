#![feature(unboxed_closures, fn_traits)]

use liquid_lang as liquid;

#[liquid::interface]
mod entry {
    extern "liquid" {
        fn getInt(key: String) -> i256;
        fn getUint(key: String) -> u256;
        fn getAddress(key: String) -> Address;
        fn getString(key: String) -> String;

        fn set(key: String, value: i256);
        fn set(key: String, value: u256);
        fn set(key: String, value: Address);
        fn set(key: String, value: String);
    }
}

#[liquid::interface]
mod kv_table {
    use super::entry::*;

    extern "liquid" {
        fn get(primary_key: String) -> (bool, Entry);
        fn set(primary_key: String, entry: Entry) -> i256;
        fn newEntry() -> Entry;
    }
}

#[liquid::interface]
mod kv_table_factory {
    use super::kv_table::*;

    extern "liquid" {
        fn openTable(name: String) -> KvTable;
        fn createTable(name: String, primary_key: String, fields: String) -> i256;
    }
}

#[liquid::contract(version = "0.2.0")]
mod kv_table_test {
    use super::kv_table_factory::*;
    use liquid_core::storage;

    #[liquid(storage)]
    struct KvTableTest {
        table_factory: storage::Value<KvTableFactory>,
    }

    #[liquid(event)]
    struct SetResult {
        count: i256,
    }

    #[liquid(methods)]
    impl KvTableTest {
        const TABLE_NAME: &'static str = "t_kvtest";

        pub fn new(&mut self) {
            self.table_factory
                .initialize(KvTableFactory::at(Address::from("0x1010")));
            self.table_factory.createTable(
                String::from(Self::TABLE_NAME),
                String::from("id"),
                String::from("item_price,item_name"),
            );
        }

        pub fn get(&self, id: String) -> (bool, i256, String) {
            let table = self
                .table_factory
                .openTable(String::from(Self::TABLE_NAME))
                .unwrap();
            let (ok, entry) = table.get(id).unwrap();
            let (item_price, item_name) = if ok {
                (
                    entry.getInt("item_price".to_string()).unwrap(),
                    entry.getString("item_name".to_string()).unwrap(),
                )
            } else {
                (0.into(), Default::default())
            };

            (ok, item_price, item_name)
        }

        pub fn set(&mut self, id: String, item_price: i256, item_name: String) -> i256 {
            let table = self
                .table_factory
                .openTable(String::from(Self::TABLE_NAME))
                .unwrap();
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
}

fn main() {}
