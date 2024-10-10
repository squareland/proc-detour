extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, FnArg, GenericParam, ItemFn, Token};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;

#[proc_macro_attribute]
pub fn detour(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let fn_ident = input.sig.ident.clone();
    let fn_inputs = input.sig.inputs.clone();
    let fn_arg_types = input.sig.inputs.iter().filter_map(|arg| {
        match arg {
            FnArg::Receiver(r) => None,
            FnArg::Typed(t) => Some(
                t.ty.clone()
            )
        }
    }).collect::<Punctuated<_, Token![,]>>();
    let fn_arg_names = input.sig.inputs.iter().filter_map(|arg| {
        match arg {
            FnArg::Receiver(r) => None,
            FnArg::Typed(t) => Some(
                t.pat.clone()
            )
        }
    }).collect::<Punctuated<_, Token![,]>>();
    let fn_output = input.sig.output.clone();
    let fn_unsafety = input.sig.unsafety.clone();
    let fn_abi = input.sig.abi.clone();
    let fn_lifetime_bounds = input.sig.generics.params.iter().filter_map(|param| {
        if let GenericParam::Lifetime(lifetime_def) = param {
            Some(lifetime_def.lifetime.clone())
        } else {
            None
        }
    });
    let fn_name = fn_ident.to_string();
    let vis = input.vis.to_token_stream();
    let noop_error = format!("detour {} is not installed", fn_name);
    let install_error = format!("detour {} installation failed", fn_name);
    let enable_error = format!("detour {} enabling failed", fn_name);

    let signature = quote! {
        for<#(#fn_lifetime_bounds),*> #fn_unsafety #fn_abi fn(#fn_arg_types) #fn_output
    };
    let fn_lifetime_bounds = input.sig.generics.params.iter().filter_map(|param| {
        if let GenericParam::Lifetime(lifetime_def) = param {
            Some(lifetime_def.lifetime.clone())
        } else {
            None
        }
    });

    let tokens = quote! {
        #input

        #vis mod #fn_ident {
            use super::*;
            use detour::RawDetour;

            pub unsafe fn install(source: #signature) {
                let d = RawDetour::new(std::mem::transmute(source), #fn_ident as _)
                    .expect(#install_error);
                d.enable().expect(#enable_error);

                let trampoline = std::mem::transmute(d.trampoline());
                std::mem::forget(d);
                SUPER = trampoline;
            }

            #[inline(never)]
            fn noop() -> ! {
                panic!(#noop_error);
            }

            type Header = #signature;
            static mut SUPER: Header = unsafe { std::mem::transmute(noop as *const std::ffi::c_void) };

            #[inline(always)]
            pub #fn_unsafety fn direct<#(#fn_lifetime_bounds),*>(#fn_inputs) #fn_output {
                unsafe {
                    SUPER(#fn_arg_names)
                }
            }
        }
    };

    tokens.into()
}
