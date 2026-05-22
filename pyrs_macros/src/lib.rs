extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input, token::Token};

#[proc_macro_derive(Builder)]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", name), name.span());

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields_named) = &data.fields {
            fields_named.named.iter()
        } else {
            unimplemented!();
        }
    } else {
        unimplemented!();
    };

    let field_defs = fields
        .clone()
        .map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            quote! { #name: Option<#ty> }
        })
        .collect::<Vec<_>>();

    let setters = fields
        .clone()
        .map(|f| {
            let name = &f.ident;
            let ty = &f.ty;
            quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#name = Some(value);
                    self
                }
            }
        })
        .collect::<Vec<_>>();

    let field_names = fields
        .map(|f| {
            let name = &f.ident;
            quote! {
                #name
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        pub struct #builder_name {
            #(#field_defs,)*
        }

        impl #builder_name {
            #(#setters)*

            pub fn build(self) -> Result<#name, &'static str> {
                Ok(#name {
                    #(#field_defs : self.#field_defs.clone().ok_or("Missing field")?,)*
                })
            }
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#field_defs : None,)*
                }
            }
        }
    };

    println!("Generated Code: \n{}", expanded);
    TokenStream::from(expanded)
}

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn hello() {
                println!("Hello, my name is {}!", stringify!(#name));
            }
        }
    };

    TokenStream::from(expanded)
}
