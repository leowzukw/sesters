/*
Sesters: easily convert one currency to another
Copyright (C) 2018-2019  Clément Joly <oss+sesters@131719.xyz>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#[cfg(test)]
const SHORT_TXT: &str = "short";
const LONG_TXT: &str = "some quite loooooooooooooooooooooooooooong text";

mod iso {
    use super::super::iso;
    use crate::currency::*;

    fn test_iso_usd_then_with_other(
        txt: &str,
        exp1_usd: &Option<PriceTag>,
        exp2_eur: &Option<PriceTag>,
        exp3_btc: &Option<PriceTag>,
    ) {
        let exp1_usd: &Option<&PriceTag> = &exp1_usd.as_ref();
        let exp2_eur: &Option<&PriceTag> = &exp2_eur.as_ref();
        let exp3_btc: &Option<&PriceTag> = &exp3_btc.as_ref();
        // Infer combination when there is at most one Some
        // let exp2 = exp1_usd.as_ref().or(exp2_eur.as_ref());
        let exp = match (exp1_usd, exp2_eur, exp3_btc) {
            (Some(ref e), None, None) => Some(e),
            (None, Some(ref e), None) => Some(e),
            (None, None, Some(ref e)) => Some(e),
            (None, None, None) => None,
            _ => panic!("More than one value is Some"),
        };
        println!("===============================");
        assert_eq!(&iso(&[USD], txt).first(), exp1_usd);
        println!("usd ok");
        assert_eq!(&iso(&[EUR], txt).first(), exp2_eur);
        println!("eur ok");
        assert_eq!(&iso(&[BTC], txt).first(), exp3_btc);
        println!("btc ok");
        assert_eq!(&iso(&[USD, EUR, BTC], txt).first(), &exp.cloned());
        println!("usd, eur, btc ok");
        println!("===============================");
    }

    #[test]
    fn iso_empty_string() {
        test_iso_usd_then_with_other(&format!(""), &None, &None, &None);
    }

    #[test]
    fn iso_none() {
        test_iso_usd_then_with_other(&format!("13"), &None, &None, &None);
    }

    #[test]
    fn iso_none_before() {
        test_iso_usd_then_with_other(&format!("OOO 13"), &None, &None, &None);
    }

    #[test]
    fn iso_none_after() {
        test_iso_usd_then_with_other(&format!("13 OOO"), &None, &None, &None);
    }

    #[test]
    fn iso_eur_before() {
        let currency_amount = Some(PriceTag::new(&EUR, 15.));
        test_iso_usd_then_with_other("EUR 15", &None, &currency_amount, &None);
    }

    /* TODO , Separator
    #[test]
    fn iso_eur_before_float() {
        let eur = EUR;
        let currency_amount = Some(PriceTag::new(&eur, 15.11));
        test_iso_usd_then_with_other("EUR 15,11", &None, &currency_amount, &None);
    }
    */

    #[test]
    fn iso_before() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, 13.));
        test_iso_usd_then_with_other("USD 13", &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_float() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, 13.5));
        test_iso_usd_then_with_other("USD 13.5", &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_null_amount() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, 0.));
        test_iso_usd_then_with_other(&format!("USD 0"), &currency_amount, &None, &None);
    }

    #[test]
    fn iso_before_negative_amount() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, -12.));
        test_iso_usd_then_with_other(&format!("USD -12"), &currency_amount, &None, &None);
    }

    /*
    #[test]
    fn iso_after() {
        let usd = USD;
        let currency_amount = Some(PriceTag::new(&usd, 13.));
        test_iso_usd_then_with_other(
            &format!("13 USD"),
            &currency_amount,
            &None,
            &currency_amount,
        );
    }
    */

    /* TODO Other cases
    #[test]
    fn iso_before_long() {
        test_iso_usd_then_with_other(&format!("USD some quite looooooong text 13"), None, None);
    }

    #[test]
    fn iso_after_long() {
        test_iso_usd_then_with_other(&format!("13 some quite loooong USD"), None, None);
    }

    #[test]
    fn iso_before_words() {
        test_iso_usd_then_with_other(&format!("USD 13"), None, None);
    }

    #[test]
    fn iso_after_words() {
        test_iso_usd_then_with_other(&format!("USD 13"), None, None);
    }
    */
}

mod price_tag_match {
    use super::super::PriceTagMatch;
    use crate::currency::{EUR, USD, BTC};

    #[test]
    fn price_tag_match_right_partial_order() {
        let a1 = PriceTagMatch::new(1.0, &EUR, 0, true);
        let a2 = PriceTagMatch::new(1.0, &USD, 0, true);
        let a3 = PriceTagMatch::new(3.0, &USD, 0, true);
        let a4 = PriceTagMatch::new(3.0, &EUR, 0, true);
        let a5 = PriceTagMatch::new(-1.0, &EUR, 0, true);
        let a6 = PriceTagMatch::new(-1.0, &BTC, 0, true);
        let b1 = PriceTagMatch::new(1.0, &EUR, 0, false);
        let b2 = PriceTagMatch::new(1.0, &USD, 0, false);
        let b3 = PriceTagMatch::new(3.0, &USD, 0, false);
        let b4 = PriceTagMatch::new(3.0, &EUR, 0, false);
        let b5 = PriceTagMatch::new(-1.0, &EUR, 0, false);
        let b6 = PriceTagMatch::new(-1.0, &BTC, 0, false);
        let c1 = PriceTagMatch::new(1.0, &EUR, 1, true);
        let c2 = PriceTagMatch::new(1.0, &USD, 1, true);
        let c3 = PriceTagMatch::new(3.0, &USD, 1, true);
        let c4 = PriceTagMatch::new(3.0, &EUR, 1, true);
        let c5 = PriceTagMatch::new(-1.0, &EUR, 1, true);
        let c6 = PriceTagMatch::new(-1.0, &BTC, 1, true);
        let d1 = PriceTagMatch::new(1.0, &EUR, 1, false);
        let d2 = PriceTagMatch::new(1.0, &USD, 1, false);
        let d3 = PriceTagMatch::new(3.0, &USD, 1, false);
        let d4 = PriceTagMatch::new(3.0, &EUR, 1, false);
        let d5 = PriceTagMatch::new(-1.0, &EUR, 1, false);
        let d6 = PriceTagMatch::new(-1.0, &BTC, 1, false);

        let v = vec![
            a1, a2, a3, a4, a5, a6,
            b1, b2, b3, b4, b5, b6,
            c1, c2, c3, c4, c5, c6,
            d1, d2, d3, d4, d5, d6,
        ];
        // TODO Use assert!(v.is_sorted()); once in stable
        for i in 0..v.len()-1 {
            assert!(v[i]<v[i+1] || (!(v[i] > v[i+1] && v[i] != v[i+1])));
        }
    }
}
