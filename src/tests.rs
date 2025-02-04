use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::*;

const S1: &[&str] = &[
    "CLASS DESCRIPTION",
    "ID NO",
    "SSN",
    "SUBSCRIBER NAME",
    "MEMBER NAME",
    "MEMBER COUNT",
    "COVERAGE TYPE",
    "COVERAGE PERIOD",
    "MEMBER LEVEL AMOUNT",
    "TOTAL AMOUNT PER SUBSCRIBER",
    "TOTAL AMOUNT PER CLASS",
];

const S2: &[&str] = &[
    "MEMBER NAME",
    "",
    "",
    "",
    "",
    "",
    "SSN",
    "",
    "",
    "MEMBER ID #",
    "",
    "",
    "",
    "",
    "",
    "",
    "TIER",
    "",
    "",
    "PLAN",
    "",
    "",
    "",
    "",
    "OFFICE ID",
    "",
    "",
    "",
    "",
    "",
    "RATE",
];

const S3: &[&str] = &[
    "Member ID No.",
    "Subscriber Name",
    "Product",
    "Volume",
    "Contract Type",
    "Number Covered",
    "Rate Chg*",
    "Subscriber Amount",
    "Dependent Amount",
    "Premium Amount",
];

const THRESHOLD: f64 = 0.7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, strum_macros::Display)]
pub enum TestColumnKind {
    MemberId,
    SubscriberName,
    Ssn,
    Plan,
    Volume,
    Tier,
    EmployeeAmount,
    DependentAmount,
    Premium,
}

impl TestColumnKind {
    pub fn header_dict(&self) -> &'static [&'static str] {
        use TestColumnKind::*;
        match self {
            MemberId => &["member id"],
            SubscriberName => &["subscriber name", "first name", "last name", "member name"],
            Ssn => &["ssn", "social security number"],
            Plan => &["plan", "product"],
            Volume => &["volume", "total amount"],
            Tier => &["tier", "type", "coverage type", "coverage"],
            EmployeeAmount => &["employee amount"],
            DependentAmount => &["dependent amount"],
            Premium => &["premium", "premium amount"],
        }
    }

    pub fn value_dict(&self) -> &'static [&'static str] {
        use TestColumnKind::*;
        const MONETARY: &[&str] = &["$0", "0$", "$0.0", "0.0$"];
        match self {
            MemberId => &["0", "0a", "a0", "0a0", "a0a"],
            SubscriberName => &["a", "a a", "a, a", "a a a", "a, a a", "a a, a", "a, a, a"],
            Ssn => &[
                "000000000",
                "000-00-0000",
                "000 00 0000",
                "000-00-aaaa",
                "000 00 aaaa",
            ],
            Plan => &["a", "a0", "a 0", "a0a", "0a0", "0a"],
            Volume => MONETARY,
            Tier => &[
                "ee",
                "employee",
                "sp",
                "spouse",
                "dp",
                "dependent",
                "fam",
                "family",
                "ech",
                "employee+child",
                "spouse+child",
                "employee+spouse",
            ],
            EmployeeAmount => MONETARY,
            DependentAmount => MONETARY,
            Premium => MONETARY,
        }
    }

    pub fn value_assessor(&self) -> SimpleAssessor {
        use TestColumnKind::*;
        let alpha_reduced = SimpleAssessor {
            is_alpha_reduced: true,
            ..Default::default()
        };
        let no_numeric_reduced = SimpleAssessor {
            is_number_reduced: false,
            ..Default::default()
        };
        let alphanum_reduced = SimpleAssessor {
            is_alpha_reduced: true,
            is_number_reduced: true,
            ..Default::default()
        };

        match self {
            MemberId => alpha_reduced,
            Ssn => no_numeric_reduced,
            Plan => alphanum_reduced,
            _ => Default::default(),
        }
    }
}

fn test_s(s: &[&str]) {
    let mut bests = Vec::with_capacity(TestColumnKind::iter().count());
    let assessor = SimpleAssessor::default();

    for variant in TestColumnKind::iter() {
        let mut max = 0.0;
        let mut best_value = "";
        for substring in s {
            let score = assessor.with_dict(substring, variant.header_dict().iter().copied());
            println!("{variant}: {substring} -> {score}");

            if score > max {
                max = score;
                best_value = substring;
            }
        }

        bests.push((best_value, max));
    }

    println!("\nBest matches:");
    for (variant, (best_value, max)) in TestColumnKind::iter().zip(bests.iter()) {
        println!("{variant}: {best_value} -> {max}");
    }

    println!("\nBest matches above threshold:");
    for (variant, (best_value, max)) in TestColumnKind::iter().zip(bests) {
        if max >= THRESHOLD {
            println!("{variant}: {best_value} -> {max}");
        }
    }
}

#[test]
fn s1() {
    test_s(S1);
}

#[test]
fn s2() {
    test_s(S2);
}

#[test]
fn s3() {
    test_s(S3);
}
