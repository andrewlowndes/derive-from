
#[cfg(test)]
mod tests {
    use derive_from::From;
    use std::fmt::Debug;

    #[derive(Debug)]
    struct A {
        the_same: String,
        first_name: String,
        age: i32
    }

    #[derive(Debug)]
    struct B {
        the_same: String,
        other_name: String,
        age: i32,
    }

    #[derive(Debug, From)]
    #[from(A, B)]
    struct C {
        the_same: String,
        #[from(overrides=(A=(rename="first_name"), B=(rename="other_name", map="lowercase")))]
        name: String,
        #[from(rename="age")]
        new_age: i32,
        #[from(skip)]
        brand_new: String,
        #[from(skip, default="String::from(\"here you go\")")]
        another: String,
    }

    fn lowercase(str: String) -> String {
        str.to_lowercase()
    }

    #[test]
    fn test_main() {
        let a = A { the_same: "A".to_string(), first_name: "Test".to_string(), age: 32 };
    
        let c: C = a.into();
        assert_eq!(c.the_same, "A");
        assert_eq!(c.name, "Test");
        assert_eq!(c.new_age, 32);
        assert_eq!(c.brand_new, "");
        assert_eq!(c.another, "here you go");

        let b = B { the_same: "B".to_string(), other_name: "Joe".to_string(), age: 34 };

        let c2: C = b.into();
        assert_eq!(c2.the_same, "B");
        assert_eq!(c2.name, "joe");
        assert_eq!(c2.new_age, 34);
    }
}
