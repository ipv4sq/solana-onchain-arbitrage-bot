// Check if a value is in a list of items
#[macro_export]
macro_rules! in_list {
    ($value:expr, $($item:expr),+) => {
        {
            let v = &$value;
            $(v == &$item)||+
        }
    };
}

// Check if a value is NOT in a list of items
#[macro_export]
macro_rules! not_in {
    ($value:expr, $($item:expr),+) => {
        !$crate::in_list!($value, $($item),+)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_in_list() {
        let x = 5;
        assert!(in_list!(x, 3, 4, 5, 6));
        assert!(!in_list!(x, 1, 2, 3));
        
        let s = "hello";
        assert!(in_list!(s, "world", "hello", "rust"));
        assert!(!in_list!(s, "world", "rust"));
    }
    
    #[test]
    fn test_not_in() {
        let x = 10;
        assert!(not_in!(x, 1, 2, 3));
        assert!(!not_in!(x, 5, 10, 15));
    }
    
    #[test]
    fn test_with_references() {
        let a = String::from("test");
        let b = String::from("hello");
        let c = String::from("world");
        
        assert!(in_list!(a, a.clone(), b.clone()));
        assert!(not_in!(a, b, c));
    }
    

}