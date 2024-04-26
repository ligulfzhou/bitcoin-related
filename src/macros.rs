#[macro_export]
macro_rules! define_table {
    ($name: ident, $key: ty, $value: ty) => {
        const $name: TableDefinition<$key, $value> = TableDefinition::new(stringify!($name));
    };
}

#[macro_export]
macro_rules! define_multimap_table {
    ($name: ident, $key: ty, $value: ty) => {
        const $name: MultimapTableDefinition<$key, $value> =
            MultimapTableDefinition::new(stringify!($name));
    };
}
