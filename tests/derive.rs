
#[cfg(test)]
mod tests {
    use derive_from_ext::From;

    #[test]
    fn test_simple() {
        struct A {
            name: String
        }

        #[derive(From)]
        #[from(A)]
        struct B {
            name: String
        }

        let a = A { name: "myname".to_string() };
        let b: B = a.into();

        assert_eq!(b.name, "myname");
    }

    #[test]
    fn test_fullname() {
        mod other {
            pub struct A {
                pub name: String
            }
        }

        #[derive(From)]
        #[from(other::A)]
        struct B {
            name: String
        }

        let a = other::A { name: "myname".to_string() };
        let b: B = a.into();

        assert_eq!(b.name, "myname");
    }
    
    #[test]
    fn test_rename() {
        struct A {
            name: String
        }

        #[derive(From)]
        #[from(A)]
        struct B {
            #[from(rename = "name")]
            new_name: String
        }

        let a = A { name: "myname".to_string() };
        let b: B = a.into();

        assert_eq!(b.new_name, "myname");
    }

    #[test]
    fn test_skip() {
        struct A {
            name: String
        }

        #[derive(From)]
        #[from(A)]
        struct B {
            name: String,
            #[from(skip)]
            other: String,
        }

        let a = A { name: "myname".to_string() };
        let b: B = a.into();

        assert_eq!(b.name, "myname");
        assert_eq!(b.other, "");
    }

    #[test]
    fn test_default() {
        struct A {
            name: String
        }

        #[derive(From)]
        #[from(A)]
        struct B {
            name: String,
            #[from(skip, default="String::from(\"Something else\")")]
            other: String,
        }

        let a = A { name: "myname".to_string() };
        let b: B = a.into();

        assert_eq!(b.name, "myname");
        assert_eq!(b.other, "Something else");
    }

    #[test]
    fn test_multiple_attributes() {
        struct A {
            name: String
        }

        #[derive(From)]
        #[from(A)]
        struct B {
            name: String,
            #[from(skip)]
            #[allow(dead_code)]
            #[deprecated]
            other: String,
        }

        let a = A { name: "myname".to_string() };
        let b: B = a.into();

        assert_eq!(b.name, "myname");
    }

    #[test]
    fn test_map() {
        fn lowercase(str: String) -> String {
            str.to_lowercase()
        }

        struct A {
            name: String
        }

        #[derive(From)]
        #[from(A)]
        struct B {
            #[from(map="lowercase")]
            name: String,
        }

        let a = A { name: "MyName".to_string() };
        let b: B = a.into();

        assert_eq!(b.name, "myname");
    }

    #[test]
    fn test_multiple_overrides() {
        fn lowercase(str: String) -> String {
            str.to_lowercase()
        }

        struct A {
            first_name: String
        }

        mod other {
            pub struct B {
                pub other_name: String
            }
        }

        #[derive(From)]
        #[from(A, other::B)]
        struct C {
            #[from(overrides=(A=(rename="first_name"), other::B=(rename="other_name", map="lowercase")))]
            name: String,
        }

        let a = A { first_name: "MyName".to_string() };
        let b = other::B { other_name: "SOMETHING ELSE".to_string() };
        
        let c1: C = a.into();
        let c2: C = b.into();

        assert_eq!(c1.name, "MyName");
        assert_eq!(c2.name, "something else");
    }
}
