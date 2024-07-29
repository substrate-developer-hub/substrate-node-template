use ahash::RandomState;
use syn::{parse_quote, visit_mut::VisitMut};

pub struct CleanDocComments;

impl CleanDocComments {
    pub fn new() -> Self {
        Self
    }
}

impl VisitMut for CleanDocComments {
    fn visit_attribute_mut(&mut self, attr: &mut syn::Attribute) {
        if attr.path().is_ident("doc") {
            *attr = parse_quote!(#[doc = ""]);
        }
        if attr.path().is_ident("freeze_struct") {
            *attr = parse_quote!(#[freeze_struct]);
        }
        syn::visit_mut::visit_attribute_mut(self, attr);
    }
}

pub fn generate_hash<T: Into<syn::Item> + Clone>(item: &T) -> u64 {
    let item = item.clone();

    // Define a fixed seed
    const SEED1: u64 = 0x12345678;
    const SEED2: u64 = 0x87654321;

    // use a fixed seed for predictable hashes
    let fixed_state = RandomState::with_seeds(SEED1, SEED2, SEED1, SEED2);

    // hash item
    let item = Into::<syn::Item>::into(item);
    fixed_state.hash_one(&item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Item;

    #[test]
    fn test_clean_doc_comments() {
        // Example code with doc comments
        let item: Item = parse_quote! {
            /// This is a doc comment
            #[cfg(feature = "example")]
            fn example() {
                println!("Hello, world!");
            }
        };

        let hash_before = generate_hash(&item);

        let mut item_clone = item.clone();
        let mut cleaner = CleanDocComments;
        cleaner.visit_item_mut(&mut item_clone);

        // Calculate the hash of the cleaned item
        let hash_after = generate_hash(&item_clone);

        assert_ne!(hash_before, hash_after);

        let item2: Item = parse_quote! {
            #[doc = ""]
            #[cfg(feature = "example")]
            fn example() {
                println!("Hello, world!");
            }
        };

        assert_eq!(hash_after, generate_hash(&item2));
    }

    #[test]
    fn test_clean_doc_comments_struct() {
        // Example code with doc comments in a struct
        let item: Item = parse_quote! {
            /// Another doc comment
            struct MyStruct {
                #[cfg(feature = "field")]
                field1: i32,
                /// Field doc comment
                field2: String,
            }
        };

        let hash_before = generate_hash(&item);

        let mut item_clone = item.clone();
        let mut cleaner = CleanDocComments;
        cleaner.visit_item_mut(&mut item_clone);

        // Calculate the hash of the cleaned item
        let hash_after = generate_hash(&item_clone);

        assert_ne!(hash_before, hash_after);

        let item2: Item = parse_quote! {
            #[doc = ""]
            struct MyStruct {
                #[cfg(feature = "field")]
                field1: i32,
                #[doc = ""]
                field2: String,
            }
        };

        assert_eq!(hash_after, generate_hash(&item2));
    }
}
