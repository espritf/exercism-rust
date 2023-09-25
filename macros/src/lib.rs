#[macro_export]
macro_rules! hashmap {
    () => { ::std::collections::HashMap::new() };
    ($($key:expr => $val:expr),+$(,)?) => {
        ::std::collections::HashMap::from([
            $(($key, $val),)*
        ])
    };
}
