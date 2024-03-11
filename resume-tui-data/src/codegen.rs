use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{punctuated::Punctuated, LitStr, Token};

use crate::{Database, Detail, Workplace};

impl From<Database<String>> for Database<LitStr> {
    fn from(value: Database<String>) -> Self {
        Database {
            name: LitStr::new(&value.name, Span::call_site()),
            tag_line: LitStr::new(&value.tag_line, Span::call_site()),
            jobs: value
                .jobs
                .into_iter()
                .map(|j| Workplace {
                    details: j
                        .details
                        .into_iter()
                        .map(|d| Detail {
                            short: LitStr::new(&d.short, Span::call_site()),
                            long: LitStr::new(&d.long, Span::call_site()),
                            detail: LitStr::new(&d.detail, Span::call_site()),
                        })
                        .collect(),
                    end: j.end.map(|s| LitStr::new(&s, Span::call_site())),
                    start: LitStr::new(&j.start, Span::call_site()),
                    name: LitStr::new(&j.name, Span::call_site()),
                    title: LitStr::new(&j.title, Span::call_site()),
                })
                .collect(),
            education: ::std::collections::LinkedList::new(),
            open_source: ::std::collections::LinkedList::new(),
        }
    }
}

impl From<Database<LitStr>> for TokenStream {
    fn from(value: Database<LitStr>) -> TokenStream {
        let Database {
            name,
            tag_line,
            jobs,
            education: _,
            open_source: _,
        } = value;
        let mut puncted: Punctuated<TokenStream, Token![,]> = Punctuated::new();
        for job in jobs.into_iter().map(TokenStream::from) {
            puncted.push(job);
        }

        quote::quote! {
            Database {
                name: #name,
                tag_line: #tag_line,
                jobs: [#puncted].into_iter().collect(),
                education: Default::default(),
                open_source: Default::default(),
            }
        }
    }
}

impl From<Workplace<LitStr>> for TokenStream {
    fn from(value: Workplace<LitStr>) -> Self {
        let Workplace {
            name,
            title,
            start,
            end,
            details,
        } = value;
        let mut puncted: Punctuated<TokenStream, Token![,]> = Punctuated::new();
        for detail in details.into_iter().map(TokenStream::from) {
            puncted.push(detail);
        }
        quote::quote! {
            Workplace {
                name: #name,
                title: #title,
                start: #start,
                end: #end,
                details: [#puncted].into_iter().collect(),
            }
        }
    }
}

impl From<Detail<LitStr>> for TokenStream {
    fn from(value: Detail<LitStr>) -> Self {
        let Detail {
            short,
            long,
            detail,
        } = value;
        quote::quote! {
            Detail {
                short: #short,
                long: #long,
                detail: #detail,
            }
        }
    }
}
