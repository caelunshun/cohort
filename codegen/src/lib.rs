extern crate proc_macro;

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro2::{TokenStream, Span};
use syn::{FnArg, Ident, ItemFn, Pat, PatType, Type};

#[proc_macro_attribute]
pub fn system(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: ItemFn = parse_macro_input!(input as ItemFn);

    let sig = &input.sig;
    assert!(
        sig.generics.params.is_empty(),
        "systems may not have generic parameters"
    );

    // Vector of `with_xx` to call on the `SystemBuilder`.
    let mut builder_args = vec![];
    // Vector of resource variable names (`Ident`s).
    let mut resource_idents = vec![];
    let mut resource_types = vec![];
    // Vector of query variable names (`Ident`s).
    //let mut query_idents = vec![];
    // Ident of the CommandBuffer variable.
    let mut cmd_buf_ident = None;
    // Ident of the PreparedWorld variable.
    let mut prep_world_ident = None;

    // Parse function arguments and determine whether they refer to resources,
    // the `PreparedWorld`, or the `CommandBuffer`.
    // Note that queries are performed inside the function using `cohort::query`.
    // This is implemented below.
    for param in &sig.inputs {
        let arg = arg(param);
        let ident = match &*arg.pat {
            Pat::Ident(ident) => ident.ident.clone(),
            _ => panic!(),
        };

        let (mutable, ty) = parse_arg(arg);

        match ty {
            ArgType::CommandBuffer => cmd_buf_ident = Some(ident),
            ArgType::PreparedWorld => prep_world_ident = Some(ident),
            ArgType::Resource(res) => {
                resource_idents.push(ident);
                let builder_arg = if mutable {
                    quote! {
                        .write_resource::<#res>()
                    }
                } else {
                    quote! {
                        .read_resource::<#res>()
                    }
                };
                builder_args.push(builder_arg);

                let ty = if mutable {
                    quote! {
                        legion::resource::PreparedRead<#res>
                    }
                } else {
                    quote! {
                        legion::resource::PreparedWrite<#res>
                    }
                };
                resource_types.push(ty);
            }
        }
    }

    // TODO: queries

    let cmd_buf_ident = cmd_buf_ident.unwrap_or(Ident::new("cmd_buf", Span::call_site()));
    let prep_world_ident = prep_world_ident.unwrap_or(Ident::new("world", Span::call_site()));

    let content = &input.block;

    let fn_ident = input.sig.ident;

    let res = quote! {
        fn #fn_ident() -> Box<dyn legion::system::Schedulable> {
            legion::system::SystemBuilder::<()>::new("")
                #(# builder_args )*
                .build(|#cmd_buf_ident, #prep_world_ident, (#( #resource_idents ),*), _: &mut ()| {
                    #content
                })
        }
    };

    res.into()
}

fn parse_arg(arg: &PatType) -> (bool, ArgType) {
    let arg = match &*arg.ty {
        Type::Reference(r) => r,
        _ => panic!("Invalid argument type"),
    };

    let mutable = arg.mutability.is_some();

    let inner = match &*arg.elem {
        Type::Path(path) => path,
        _ => panic!("Invalid argument type"),
    };

    let ty = inner.path.segments.last().expect("no last path segment");

    let prepared_world = "PreparedWorld";
    let command_buffer = "CommandBuffer";

    let string = ty.ident.to_string();

    let ty = if &string == prepared_world {
        ArgType::PreparedWorld
    } else if &string == command_buffer {
        ArgType::CommandBuffer
    } else {
        let ty = &inner.path;
        ArgType::Resource(quote! { #ty })
    };

    (mutable, ty)
}

enum ArgType {
    PreparedWorld,
    CommandBuffer,
    Resource(TokenStream),
}

fn arg(arg: &FnArg) -> &PatType {
    match arg {
        FnArg::Typed(ty) => ty,
        _ => panic!("systems may not accept `self` parameters"),
    }
}
