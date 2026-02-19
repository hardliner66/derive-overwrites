use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ImplItem, ItemImpl, LitStr, Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

#[proc_macro_attribute]
pub fn skip(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn overwrite(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn generate_overwrites(attr: TokenStream, item: TokenStream) -> TokenStream {
    let GenerateArgs {
        all,
        passthrough,
        name,
    } = syn::parse_macro_input!(attr as GenerateArgs);
    let impl_block = parse_macro_input!(item as ItemImpl);

    let self_ty = &impl_block.self_ty;

    let struct_name = match self_ty.as_ref() {
        Type::Path(type_path) => type_path.path.segments.last().map(|seg| &seg.ident),
        _ => None,
    };

    let trait_name = if let Some(name) = name {
        Some(syn::Ident::new(&name, struct_name.span()))
    } else {
        struct_name.map(|name| syn::Ident::new(&format!("{name}Overwrites"), name.span()))
    };

    let mut trait_methods = Vec::new();

    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            let is_public = matches!(method.vis, syn::Visibility::Public(_));

            let has_ignore = method.attrs.iter().any(|attr| attr.path().is_ident("skip"));

            let has_overwrite = method
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("overwrite"));

            if is_public && ((all && !has_ignore) || (!all && has_overwrite)) {
                let sig = &method.sig;
                let attrs = &method.attrs;

                trait_methods.push(quote! {
                    #(#attrs)*
                    #sig;
                });
            }
        }
    }

    let impl_methods = if passthrough {
        let mut impl_methods = Vec::new();

        for item in &impl_block.items {
            if let ImplItem::Fn(method) = item {
                let is_public = matches!(method.vis, syn::Visibility::Public(_));
                let has_ignore = method.attrs.iter().any(|attr| attr.path().is_ident("skip"));

                let has_overwrite = method
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("overwrite"));

                if is_public && ((all && !has_ignore) || (!all && has_overwrite)) {
                    let sig = &method.sig;
                    let block = &method.block;
                    let attrs = &method.attrs;

                    impl_methods.push(quote! {
                        #(#attrs)*
                        #sig #block
                    });
                }
            }
        }
        Some(impl_methods)
    } else {
        None
    };

    let generics = &impl_block.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let trait_and_impl = if let Some(trait_name) = trait_name {
        if trait_methods.is_empty() {
            let span = format!("{:?}", impl_block.span());
            quote! {
                compile_error!(
                    "Must overwrite at least one function! {}", #span
                );
            }
        } else {
            let base = quote! {
                pub trait #trait_name #impl_generics #where_clause {
                    #(#trait_methods)*
                }
            };
            if let Some(impl_methods) = impl_methods {
                quote! {
                    #base

                    impl #impl_generics #trait_name #ty_generics for #self_ty #where_clause {
                        #(#impl_methods)*
                    }
                }
            } else {
                quote! {
                    #base
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #trait_and_impl

        #impl_block
    };

    TokenStream::from(expanded)
}

struct GenerateArgs {
    all: bool,
    passthrough: bool,
    name: Option<String>,
}

impl Parse for GenerateArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut all = true;
        let mut passthrough = false;
        let mut name = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            match ident.to_string().as_str() {
                "default" => {
                    input.parse::<Token![=]>()?;
                    let value: LitStr = input.parse()?;
                    let value_string = value.value();
                    if let "overwrite" | "skip" = value_string.as_str() {
                        all = value_string == "overwrite";
                    } else {
                        return Err(syn::Error::new(
                            input.span(),
                            "Expected 'overwrite' or 'skip'",
                        ));
                    }
                }
                "name" => {
                    input.parse::<Token![=]>()?;
                    let value: LitStr = input.parse()?;
                    name = Some(value.value());
                }
                "passthrough" => {
                    passthrough = true;
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "Unknown argument"));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(GenerateArgs {
            all,
            passthrough,
            name,
        })
    }
}
