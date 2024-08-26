//! ## Proc macro for `udigest` crate
//!
//! This crate contains a proc macro for implementing `Digestable` trait
//! from [udigest crate](https://docs.rs/udigest), please refer to its
//! documentation.

use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Error, Result};

mod attrs;

#[proc_macro_derive(Digestable, attributes(udigest))]
pub fn digestable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match digestable_inner(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn digestable_inner(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream> {
    let mut container_attrs = ContainerAttrs::default();

    // Parse container-level attributes
    for attr in input.attrs {
        let Some(attr) = parse_attribute(&attr)? else {
            continue;
        };
        match attr {
            attrs::Attr::Root(_) if container_attrs.root.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::Root(attr) => {
                container_attrs.root = Some(attr);
            }
            attrs::Attr::Tag(_) if container_attrs.tag.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::Tag(attr) => {
                container_attrs.tag = Some(attr);
            }
            attrs::Attr::Bound(_) if container_attrs.bound.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::Bound(attr) => {
                container_attrs.bound = Some(attr);
            }
            _ => return Err(Error::new(attr.kw_span(), "attribute is not allowed here")),
        }
    }

    match input.data {
        syn::Data::Struct(s) => process_struct(&container_attrs, &input.ident, &input.generics, &s),
        syn::Data::Enum(e) => process_enum(&container_attrs, &input.ident, &input.generics, &e),
        syn::Data::Union(u) => Err(Error::new(u.union_token.span, "unions are not supported")),
    }
}

fn process_enum(
    attrs: &ContainerAttrs,
    name: &syn::Ident,
    generics: &syn::Generics,
    e: &syn::DataEnum,
) -> Result<proc_macro2::TokenStream> {
    let variants = e
        .variants
        .iter()
        .map(|v| {
            Ok(Variant {
                name: v.ident.clone(),
                ty: match &v.fields {
                    syn::Fields::Named(_) => VariantType::Named,
                    syn::Fields::Unnamed(_) => VariantType::Unnamed,
                    syn::Fields::Unit => VariantType::Unit,
                },
                fields: (0..)
                    .zip(v.fields.iter())
                    .map(|(i, f)| process_field(&attrs.get_root_path(), i, f))
                    .collect::<Result<Vec<_>>>()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    generate_impl_for_enum(attrs, name, generics, &variants)
}

fn process_struct(
    container_attrs: &ContainerAttrs,
    name: &syn::Ident,
    generics: &syn::Generics,
    s: &syn::DataStruct,
) -> Result<proc_macro2::TokenStream> {
    let struct_fields = (0..)
        .zip(s.fields.iter())
        .map(|(i, f)| process_field(&container_attrs.get_root_path(), i, f))
        .collect::<Result<Vec<_>>>()?;

    generate_impl_for_struct(container_attrs, name, generics, &struct_fields)
}

fn process_field(root_path: &attrs::RootPath, index: u32, field: &syn::Field) -> Result<Field> {
    // same_ty = <root_path>::as_::Same
    let same_ty = {
        let mut root = root_path.clone();
        root.extend([
            syn::Ident::new("as_", root_path.span()),
            syn::Ident::new("Same", root_path.span()),
        ]);
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: root
                    .into_iter()
                    .map(|ident| syn::PathSegment {
                        ident,
                        arguments: syn::PathArguments::None,
                    })
                    .collect(),
            },
        })
    };
    let mut field_attrs = FieldAttrs::default();

    let mem = field
        .ident
        .clone()
        .map(syn::Member::Named)
        .unwrap_or_else(|| {
            syn::Index {
                index,
                span: field.span(),
            }
            .into()
        });

    for attr in &field.attrs {
        let Some(attr) = parse_attribute(attr)? else {
            continue;
        };
        match attr {
            attrs::Attr::AsBytes(_) if field_attrs.as_bytes.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::With(_) if field_attrs.with.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::Skip(_) if field_attrs.skip.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"));
            }
            attrs::Attr::Rename(_) if field_attrs.rename.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::As(_) if field_attrs.as_.is_some() => {
                return Err(Error::new(attr.kw_span(), "attribute is duplicated"))
            }
            attrs::Attr::AsBytes(_)
            | attrs::Attr::With(_)
            | attrs::Attr::As(_)
            | attrs::Attr::Skip(_)
                if count_trues([
                    field_attrs.as_bytes.is_some(),
                    field_attrs.with.is_some(),
                    field_attrs.as_.is_some(),
                    field_attrs.skip.is_some(),
                ]) > 0 =>
            {
                return Err(Error::new(
                    attr.kw_span(),
                    "attributes `with`, `as_bytes`, `as` and 'skip` cannot be used together",
                ));
            }
            attrs::Attr::AsBytes(attr) => {
                field_attrs.as_bytes = Some(attr);
            }
            attrs::Attr::With(attr) => {
                field_attrs.with = Some(attr);
            }
            attrs::Attr::Skip(attr) => {
                field_attrs.skip = Some(attr);
            }
            attrs::Attr::Rename(attr) => {
                field_attrs.rename = Some(attr);
            }
            attrs::Attr::As(mut attr) => {
                attr.value = type_replace_infer(attr.value, same_ty.clone())?;
                field_attrs.as_ = Some(attr);
            }
            _ => return Err(Error::new(attr.kw_span(), "attribute is not allowed here")),
        }
    }

    Ok(Field {
        span: field.ty.span(),
        attrs: field_attrs,
        mem,
        ty: field.ty.clone(),
    })
}

fn count_trues(i: impl IntoIterator<Item = bool>) -> usize {
    i.into_iter().filter(|x| *x).count()
}

/// Traverses the type and replaces `_` with `infer_ty`
///
/// E.g. `Option<_>` becomes `Option<{infer_ty}>`.
///
/// Returns an error if provided type is not supported. It supports any types that
/// can be found as the type of field in the struct. For instance, `impl Trait` is
/// not supported.
///
/// The function only traverses some types such as: path type (e.g. `std::result::Result<T, E>`),
/// arrays, slices, tuples, references, pointers. It does not traverse anything else,
/// like function pointers or trait objects. E.g. `fn(_) -> u32` or `Box<dyn _>` are
/// not modified by the function.
fn type_replace_infer(ty: syn::Type, infer_ty: syn::Type) -> Result<syn::Type> {
    match ty {
        syn::Type::Infer(_) => Ok(infer_ty),

        syn::Type::Array(ty) => Ok(syn::Type::Array(syn::TypeArray {
            bracket_token: ty.bracket_token,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
            semi_token: ty.semi_token,
            len: ty.len,
        })),
        syn::Type::Group(ty) => Ok(syn::Type::Group(syn::TypeGroup {
            group_token: ty.group_token,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Paren(ty) => Ok(syn::Type::Paren(syn::TypeParen {
            paren_token: ty.paren_token,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Path(ty) => Ok(syn::Type::Path(syn::TypePath {
            qself: ty.qself,
            path: syn::Path {
                leading_colon: ty.path.leading_colon,
                // Traverse each segment of the path, e.g.:
                //
                // std::result::Result<T, E>
                // 1 --| 2 ----| 3 --------|
                segments: ty
                    .path
                    .segments
                    .into_pairs()
                    .map(|pair| {
                        let (seg, sep) = pair.into_tuple();
                        let args = match seg.arguments {
                            syn::PathArguments::None => syn::PathArguments::None,
                            syn::PathArguments::Parenthesized(x) => {
                                return Err(Error::new(x.span(), "not allowed in this context"))
                            }
                            // Result<T, E>
                            //       ^----^ angle-bracketed arguments
                            syn::PathArguments::AngleBracketed(args) => {
                                syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: args.colon2_token,
                                        lt_token: args.lt_token,
                                        // traverse each path argument
                                        args: args
                                            .args
                                            .into_pairs()
                                            .map(|pair| {
                                                let (arg, comma) = pair.into_tuple();
                                                let arg = match arg {
                                                    // type argument => need to traverse
                                                    syn::GenericArgument::Type(ty) => {
                                                        syn::GenericArgument::Type(
                                                            type_replace_infer(
                                                                ty,
                                                                infer_ty.clone(),
                                                            )?,
                                                        )
                                                    }
                                                    // other arguments we do not care about, like lifetimes
                                                    _ => arg,
                                                };
                                                Ok(syn::punctuated::Pair::new(arg, comma))
                                            })
                                            .collect::<Result<_>>()?,
                                        gt_token: args.gt_token,
                                    },
                                )
                            }
                        };

                        Ok(syn::punctuated::Pair::new(
                            syn::PathSegment {
                                ident: seg.ident,
                                arguments: args,
                            },
                            sep,
                        ))
                    })
                    .collect::<Result<_>>()?,
            },
        })),
        syn::Type::Ptr(ty) => Ok(syn::Type::Ptr(syn::TypePtr {
            star_token: ty.star_token,
            const_token: ty.const_token,
            mutability: ty.mutability,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Reference(ty) => Ok(syn::Type::Reference(syn::TypeReference {
            and_token: ty.and_token,
            lifetime: ty.lifetime,
            mutability: ty.mutability,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Slice(ty) => Ok(syn::Type::Slice(syn::TypeSlice {
            bracket_token: ty.bracket_token,
            elem: Box::new(type_replace_infer(*ty.elem, infer_ty)?),
        })),
        syn::Type::Tuple(ty) => Ok(syn::Type::Tuple(syn::TypeTuple {
            paren_token: ty.paren_token,
            // Traverse each type in the tuple
            elems: ty
                .elems
                .into_pairs()
                .map(|pair| {
                    let (ty, comma) = pair.into_tuple();
                    let ty = type_replace_infer(ty, infer_ty.clone())?;
                    Ok(syn::punctuated::Pair::new(ty, comma))
                })
                .collect::<Result<_>>()?,
        })),

        // Following types are not traversed
        syn::Type::BareFn(_)
        | syn::Type::Macro(_)
        | syn::Type::Never(_)
        | syn::Type::TraitObject(_)
        | syn::Type::Verbatim(_) => Ok(ty),

        // Following types are not supported
        syn::Type::ImplTrait(_) => Err(Error::new(
            ty.span(),
            "`impl Trait` is not supported in this context",
        )),

        // This might happen if Rust gets a new type in the future
        _ => Err(Error::new(ty.span(), "unknown type")),
    }
}

fn generate_impl_for_enum(
    attrs: &ContainerAttrs,
    enum_name: &syn::Ident,
    enum_generics: &syn::Generics,
    enum_variants: &[Variant],
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let (impl_generics, ty_generics, _) = enum_generics.split_for_impl();

    let where_clause = make_where_clause(attrs, enum_generics)?;

    let encoder_var = syn::Ident::new("encoder", proc_macro2::Span::call_site());

    let specify_tag = attrs.tag.as_ref().map(|attrs::Tag { value, .. }| {
        quote_spanned! {value.span() =>
            let tag = #value;
            let tag = AsRef::<[u8]>::as_ref(&tag);
            #encoder_var.set_tag(tag);
        }
    });

    let match_expr = if !enum_variants.is_empty() {
        let match_branches = enum_variants.iter().map(|v| {
            let variant_name = &v.name;
            let field_bindings = (0..v.fields.len())
                .map(|i| syn::Ident::new(&format!("field{i}"), proc_macro2::Span::call_site()))
                .collect::<Vec<_>>();
            let pattern = match v.ty {
                VariantType::Named => {
                    let fields = v.fields.iter().zip(&field_bindings).map(|(f, binding)| {
                        let field_name = &f.mem;
                        quote! { #field_name: #binding }
                    });
                    quote! { {#(#fields),*} }
                }
                VariantType::Unnamed => {
                    let fields = field_bindings.iter().map(|binding| {
                        quote! {#binding}
                    });
                    quote! { (#(#fields),*) }
                }
                VariantType::Unit => {
                    quote!()
                }
            };

            let encode_fields = field_bindings.iter().zip(&v.fields).map(|(binding, f)| {
                encode_field(
                    &root_path,
                    &encoder_var,
                    &f.attrs,
                    f.span,
                    &f.stringify_field_name(),
                    &f.ty,
                    &binding,
                )
            });

            let variant_name_str = variant_name.to_string();
            quote_spanned! {variant_name.span() =>
                #enum_name::#variant_name #pattern => {
                    let mut #encoder_var = #encoder_var.with_variant(#variant_name_str);
                    #(#encode_fields)*
                }
            }
        });
        quote! {
            match self {
                #(#match_branches)*
            }
        }
    } else {
        quote! {
            match *self {}
        }
    };

    Ok(quote! {
        impl #impl_generics #root_path::Digestable for #enum_name #ty_generics #where_clause {
            fn unambiguously_encode<B>(&self, encoder: #root_path::encoding::EncodeValue<B>)
            where
                B: #root_path::Buffer
            {
                let mut #encoder_var = encoder.encode_enum();
                #specify_tag
                #match_expr
            }
        }
    })
}

fn generate_impl_for_struct(
    attrs: &ContainerAttrs,
    struct_name: &syn::Ident,
    struct_generics: &syn::Generics,
    struct_fields: &[Field],
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let (impl_generics, ty_generics, _) = struct_generics.split_for_impl();

    let where_clause = make_where_clause(attrs, struct_generics)?;

    let specify_tag = attrs.tag.as_ref().map(|attrs::Tag { value, .. }| {
        quote_spanned! {value.span() =>
            let tag = #value;
            let tag = AsRef::<[u8]>::as_ref(&tag);
            encoder.set_tag(tag);
        }
    });

    let encoder_var = syn::Ident::new("encoder", proc_macro2::Span::call_site());
    let encode_each_field = struct_fields.iter().map(|f| {
        let mem = &f.mem;
        encode_field(
            &root_path,
            &encoder_var,
            &f.attrs,
            f.span,
            &f.stringify_field_name(),
            &f.ty,
            &quote_spanned! {f.ty.span() => &self.#mem},
        )
    });

    Ok(quote! {
        impl #impl_generics #root_path::Digestable for #struct_name #ty_generics #where_clause {
            fn unambiguously_encode<B>(&self, encoder: #root_path::encoding::EncodeValue<B>)
            where
                B: #root_path::Buffer
            {
                let mut #encoder_var = encoder.encode_struct();
                #specify_tag
                #(#encode_each_field)*
                #encoder_var.finish();
            }
        }
    })
}

fn parse_attribute(attr: &syn::Attribute) -> Result<Option<attrs::Attr>> {
    let attr_tokens = match &attr.meta {
        syn::Meta::List(meta) if meta.path.is_ident("udigest") => &meta.tokens,
        syn::Meta::Path(path) if path.is_ident("udigest") => {
            return Err(Error::new(
                path.span(),
                "empty attribute doesn't make sense",
            ))
        }
        syn::Meta::NameValue(meta) if meta.path.is_ident("udigest") => {
            return Err(Error::new(
                meta.value.span(),
                "attribute needs to be specified in parentheses (e.g. `#[udigest(skip)]`)",
            ));
        }
        _ => return Ok(None),
    };
    syn::parse2(attr_tokens.clone()).map(Some)
}

/// Takes the generics defined for the data type, produces a where clause that should
/// be used for trait implementation
///
/// If `bound` attribute is not specified, it takes where clause defined for datatype,
/// and populates it with constraints `A: Digestable` for every generic type defined for
/// the structure
///
/// If `bound` attribute is specified, it fully overrides the where clause
fn make_where_clause(
    attrs: &ContainerAttrs,
    generics: &syn::Generics,
) -> Result<proc_macro2::TokenStream> {
    let root_path = attrs.get_root_path();
    let predicates = generics.where_clause.as_ref().map(|w| &w.predicates);

    let generated_predicates = match &attrs.bound {
        Some(bound) => {
            let overridden_where_clause: proc_macro2::TokenStream = bound
                .value
                .value()
                .parse()
                .map_err(|err| Error::new(bound.value.span(), err))?;
            let predicates = syn::parse::Parser::parse2(
                syn::punctuated::Punctuated::<syn::WherePredicate, syn::Token![,]>::parse_terminated,
                overridden_where_clause
            )
            .map_err(|err| Error::new(bound.value.span(), err))?;
            let predicates = predicates.iter();
            quote_spanned! {bound.value.span() =>
                #(#predicates,)*
            }
        }
        None => {
            let generated_predicates = generics.type_params().map(|g| {
                let ident = &g.ident;
                quote! {#ident: #root_path::Digestable,}
            });
            quote! { #(#generated_predicates)* }
        }
    };
    Ok(quote! {
        where #generated_predicates #predicates
    })
}

/// Generates a code that encodes a field into `encoder_var`
///
/// `field_name` represents a stringified name of the field, `field_ref` contains
/// expression that yields a reference to the field. `field_span` specifies a span
/// of the field, and `field_attrs` specifies field-level attributes.
///
/// `root_path` specifies a path to the `udigest` crate.
fn encode_field(
    root_path: &attrs::RootPath,
    encoder_var: &syn::Ident,
    field_attrs: &FieldAttrs,
    field_span: proc_macro2::Span,
    field_name: &str,
    field_type: &syn::Type,
    field_ref: &impl quote::ToTokens,
) -> proc_macro2::TokenStream {
    if field_attrs.skip.is_some() {
        return quote! {};
    }

    let field_name = match &field_attrs.rename {
        None => quote! { #field_name },
        Some(attrs::Rename { rename, value, .. }) => quote_spanned! { rename.span => #value },
    };

    match (&field_attrs.as_bytes, &field_attrs.with, &field_attrs.as_) {
        (Some(attr), None, None) => match &attr.value {
            Some(func) => quote_spanned! {field_span => {
                let field_encoder = #encoder_var.add_field(#field_name);
                let field_bytes = #func(#field_ref);
                let field_bytes = AsRef::<[u8]>::as_ref(&field_bytes);
                field_encoder.encode_leaf_value(field_bytes);
            }},
            None => quote_spanned!(field_span => {
                let field_encoder = #encoder_var.add_field(#field_name);
                let field_bytes: &[u8] = AsRef::<[u8]>::as_ref(#field_ref);
                field_encoder.encode_leaf_value(field_bytes);
            }),
        },
        (None, Some(attrs::With { value: func, .. }), None) => quote_spanned! {field_span => {
            let field_encoder = #encoder_var.add_field(#field_name);
            #[allow(clippy::needless_borrow, clippy::needless_borrows_for_generic_args)]
            #func(#field_ref, field_encoder);
        }},
        (None, None, Some(attrs::As { value: ty, .. })) => quote_spanned! {field_span => {
            let field_encoder = #encoder_var.add_field(#field_name);
            #[allow(clippy::needless_borrow, clippy::needless_borrows_for_generic_args)]
            <#ty as #root_path::DigestAs<#field_type>>::digest_as(#field_ref, field_encoder)
        }},
        (None, None, None) => quote_spanned! {field_span => {
            let field_encoder = #encoder_var.add_field(#field_name);
            #root_path::Digestable::unambiguously_encode(#field_ref, field_encoder);
        }},
        _ => {
            unreachable!("it should have been validated that `with`, `as_bytes`, `as` are not used in the same time")
        }
    }
}

#[derive(Default)]
struct ContainerAttrs {
    root: Option<attrs::Root>,
    tag: Option<attrs::Tag>,
    bound: Option<attrs::Bound>,
}

impl ContainerAttrs {
    pub fn get_root_path(&self) -> attrs::RootPath {
        self.root
            .as_ref()
            .map(|root| root.path.clone())
            .unwrap_or_else(|| {
                [syn::Ident::new("udigest", proc_macro2::Span::call_site())]
                    .into_iter()
                    .collect()
            })
    }
}

#[derive(Default)]
struct FieldAttrs {
    as_bytes: Option<attrs::AsBytes>,
    skip: Option<attrs::Skip>,
    rename: Option<attrs::Rename>,
    with: Option<attrs::With>,
    as_: Option<attrs::As>,
}

struct Field {
    span: proc_macro2::Span,
    attrs: FieldAttrs,
    mem: syn::Member,
    ty: syn::Type,
}

impl Field {
    pub fn stringify_field_name(&self) -> String {
        match &self.mem {
            syn::Member::Named(ident) => ident.to_string(),
            syn::Member::Unnamed(index) => index.index.to_string(),
        }
    }
}

struct Variant {
    name: syn::Ident,
    fields: Vec<Field>,
    ty: VariantType,
}

#[derive(PartialEq, Eq)]
enum VariantType {
    Named,
    Unnamed,
    Unit,
}
