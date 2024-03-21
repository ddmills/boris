use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(TaskBuilder)]
pub fn my_macro_here_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let component_name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let label_method = label_method(&component_name);
    let insert_method = insert_method(&component_name, &ty_generics);
    let remove_method = remove_method(&component_name);

    let gen = quote! {
        impl #impl_generics TaskBuilder for #component_name #ty_generics #where_clause {
            #label_method
            #insert_method
            #remove_method
        }
    };

    proc_macro::TokenStream::from(gen)
}

fn label_method(component_name: &Ident) -> TokenStream {
    quote! {
        fn label(&self) -> String {
            String::from(stringify!(#component_name))
        }
    }
}

fn insert_method(component_name: &Ident, ty_generics: &syn::TypeGenerics) -> TokenStream {
    let turbofish = ty_generics.as_turbofish();

    quote! {
        fn insert(&self, commands: &mut ::bevy::ecs::system::Commands, actor: ::bevy::ecs::entity::Entity) {
            commands.entity(actor).insert(#component_name #turbofish::clone(self));
        }
    }
}

fn remove_method(component_name: &Ident) -> TokenStream {
    quote! {
        fn remove(&self, commands: &mut ::bevy::ecs::system::Commands, actor: ::bevy::ecs::entity::Entity) {
            commands.entity(actor).remove::<#component_name>();
        }
    }
}
