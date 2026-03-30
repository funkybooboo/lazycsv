use rand::RngExt;
use rayon::prelude::*;
use std::io::Write;

/// Supported data generation types.
const VALID_TYPES: &[&str] = &[
    "customer",
    "sales",
    "marketing",
    "weather",
    "scientific",
    "random",
];

/// Validate the generation type string.
pub fn validate_type(gen_type: &str) -> Result<(), String> {
    if VALID_TYPES.contains(&gen_type) {
        Ok(())
    } else {
        Err(format!(
            "Unknown generate type '{}'. Valid types: {}",
            gen_type,
            VALID_TYPES.join(", ")
        ))
    }
}

/// Batch size for parallel generation. Each batch is generated in parallel
/// across all cores, then written sequentially to the output.
const BATCH_SIZE: usize = 100_000;

/// Generate CSV data and write it to the given writer.
/// Uses rayon to generate rows in parallel across all available CPU cores.
pub fn generate_csv<W: Write>(
    writer: &mut W,
    rows: usize,
    cols: usize,
    gen_type: &str,
) -> anyhow::Result<()> {
    let headers = build_headers(gen_type, cols);
    let generators = build_generators(gen_type, cols);

    // Write header row
    let header_line: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    writeln!(writer, "{}", header_line.join(","))?;

    // Generate and write rows in batches using all CPU cores
    for batch_start in (0..rows).step_by(BATCH_SIZE) {
        let batch_end = (batch_start + BATCH_SIZE).min(rows);

        let batch: Vec<String> = (batch_start..batch_end)
            .into_par_iter()
            .map(|row_idx| {
                let mut rng = rand::rng();
                let mut fields = Vec::with_capacity(cols);
                for (col_idx, gen) in generators.iter().enumerate() {
                    fields.push(gen(row_idx, col_idx, &mut rng));
                }
                fields.join(",")
            })
            .collect();

        for line in &batch {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    Ok(())
}

type GenFn = Box<dyn Fn(usize, usize, &mut rand::rngs::ThreadRng) -> String + Send + Sync>;

fn build_headers(gen_type: &str, cols: usize) -> Vec<String> {
    let typed_headers: &[&str] = match gen_type {
        "customer" => &[
            "CustomerID",
            "FirstName",
            "LastName",
            "Email",
            "Phone",
            "City",
            "State",
            "ZipCode",
            "Country",
            "SignupDate",
            "Age",
            "Gender",
            "AccountBalance",
            "LoyaltyTier",
            "LastPurchase",
        ],
        "sales" => &[
            "OrderID",
            "Date",
            "Product",
            "Category",
            "Quantity",
            "UnitPrice",
            "Total",
            "CustomerID",
            "Region",
            "PaymentMethod",
            "Discount",
            "Tax",
            "ShipDate",
            "Status",
            "Channel",
        ],
        "marketing" => &[
            "CampaignID",
            "CampaignName",
            "Channel",
            "StartDate",
            "Impressions",
            "Clicks",
            "CTR",
            "Spend",
            "Conversions",
            "CPA",
            "Revenue",
            "ROI",
            "TargetAudience",
            "Region",
            "Status",
        ],
        "weather" => &[
            "Date",
            "City",
            "TempHigh",
            "TempLow",
            "Humidity",
            "WindSpeed",
            "Precipitation",
            "Condition",
            "Pressure",
            "UVIndex",
            "Visibility",
            "DewPoint",
            "CloudCover",
            "WindDirection",
            "Sunrise",
        ],
        "scientific" => &[
            "ExperimentID",
            "Timestamp",
            "SensorID",
            "Temperature",
            "Pressure",
            "Humidity",
            "Voltage",
            "Current",
            "Frequency",
            "Status",
            "BatchID",
            "SampleSize",
            "pH",
            "Concentration",
            "Duration",
        ],
        _ => &[], // random
    };

    let mut headers = Vec::with_capacity(cols);
    for i in 0..cols {
        if i < typed_headers.len() {
            headers.push(typed_headers[i].to_string());
        } else {
            headers.push(format!("C{}", i + 1));
        }
    }
    headers
}

fn build_generators(gen_type: &str, cols: usize) -> Vec<GenFn> {
    let mut generators: Vec<GenFn> = Vec::with_capacity(cols);

    match gen_type {
        "customer" => {
            let typed: Vec<GenFn> = vec![
                gen_sequential_id("CUST"),
                gen_first_name(),
                gen_last_name(),
                gen_email(),
                gen_phone(),
                gen_city(),
                gen_state(),
                gen_zip(),
                gen_country(),
                gen_date(2020, 2025),
                gen_int_range(18, 85),
                gen_pick(&["Male", "Female", "Non-binary"]),
                gen_decimal(0.0, 50000.0, 2),
                gen_pick(&["Bronze", "Silver", "Gold", "Platinum"]),
                gen_date(2023, 2025),
            ];
            for (i, g) in typed.into_iter().enumerate() {
                if i >= cols {
                    break;
                }
                generators.push(g);
            }
        }
        "sales" => {
            let typed: Vec<GenFn> = vec![
                gen_sequential_id("ORD"),
                gen_date(2023, 2025),
                gen_product(),
                gen_pick(&[
                    "Electronics",
                    "Clothing",
                    "Home",
                    "Sports",
                    "Books",
                    "Food",
                    "Toys",
                    "Office",
                ]),
                gen_int_range(1, 50),
                gen_decimal(1.0, 500.0, 2),
                gen_computed_total(4, 5), // qty * price
                gen_sequential_id("CUST"),
                gen_pick(&[
                    "North",
                    "South",
                    "East",
                    "West",
                    "Central",
                    "Northeast",
                    "Southeast",
                ]),
                gen_pick(&[
                    "Credit Card",
                    "Debit Card",
                    "PayPal",
                    "Cash",
                    "Wire Transfer",
                ]),
                gen_decimal(0.0, 0.3, 2),
                gen_decimal(0.0, 50.0, 2),
                gen_date(2023, 2025),
                gen_pick(&[
                    "Delivered",
                    "Shipped",
                    "Processing",
                    "Returned",
                    "Cancelled",
                ]),
                gen_pick(&["Online", "In-Store", "Phone", "Mobile App"]),
            ];
            for (i, g) in typed.into_iter().enumerate() {
                if i >= cols {
                    break;
                }
                generators.push(g);
            }
        }
        "marketing" => {
            let typed: Vec<GenFn> = vec![
                gen_sequential_id("CMP"),
                gen_campaign_name(),
                gen_pick(&[
                    "Email",
                    "Social Media",
                    "Search",
                    "Display",
                    "TV",
                    "Radio",
                    "Print",
                ]),
                gen_date(2023, 2025),
                gen_int_range(1000, 1_000_000),
                gen_int_range(10, 50000),
                gen_decimal(0.01, 0.15, 4),
                gen_decimal(100.0, 100000.0, 2),
                gen_int_range(1, 5000),
                gen_decimal(1.0, 200.0, 2),
                gen_decimal(100.0, 500000.0, 2),
                gen_decimal(-0.5, 10.0, 2),
                gen_pick(&[
                    "18-24", "25-34", "35-44", "45-54", "55-64", "65+", "All Ages",
                ]),
                gen_pick(&[
                    "North America",
                    "Europe",
                    "Asia",
                    "South America",
                    "Africa",
                    "Global",
                ]),
                gen_pick(&["Active", "Paused", "Completed", "Draft"]),
            ];
            for (i, g) in typed.into_iter().enumerate() {
                if i >= cols {
                    break;
                }
                generators.push(g);
            }
        }
        "weather" => {
            let typed: Vec<GenFn> = vec![
                gen_date(2023, 2025),
                gen_city(),
                gen_int_range(50, 110),
                gen_int_range(20, 80),
                gen_int_range(10, 100),
                gen_decimal(0.0, 60.0, 1),
                gen_decimal(0.0, 5.0, 2),
                gen_pick(&[
                    "Sunny",
                    "Cloudy",
                    "Rainy",
                    "Stormy",
                    "Snowy",
                    "Foggy",
                    "Windy",
                    "Partly Cloudy",
                    "Clear",
                ]),
                gen_decimal(28.0, 31.0, 2),
                gen_int_range(0, 11),
                gen_decimal(1.0, 20.0, 1),
                gen_int_range(15, 75),
                gen_int_range(0, 100),
                gen_pick(&["N", "NE", "E", "SE", "S", "SW", "W", "NW"]),
                gen_time(),
            ];
            for (i, g) in typed.into_iter().enumerate() {
                if i >= cols {
                    break;
                }
                generators.push(g);
            }
        }
        "scientific" => {
            let typed: Vec<GenFn> = vec![
                gen_sequential_id("EXP"),
                gen_timestamp(),
                gen_sequential_id("SEN"),
                gen_decimal(-50.0, 150.0, 3),
                gen_decimal(0.5, 5.0, 4),
                gen_decimal(0.0, 100.0, 2),
                gen_decimal(0.0, 24.0, 3),
                gen_decimal(0.0, 10.0, 4),
                gen_decimal(50.0, 60.0, 2),
                gen_pick(&["OK", "Warning", "Error", "Calibrating", "Offline"]),
                gen_sequential_id("BAT"),
                gen_int_range(10, 10000),
                gen_decimal(0.0, 14.0, 2),
                gen_decimal(0.0, 100.0, 4),
                gen_decimal(0.1, 3600.0, 1),
            ];
            for (i, g) in typed.into_iter().enumerate() {
                if i >= cols {
                    break;
                }
                generators.push(g);
            }
        }
        _ => {} // random - handled below
    }

    // Fill remaining columns with random generators
    while generators.len() < cols {
        let col_idx = generators.len();
        generators.push(gen_random_column(col_idx));
    }

    generators
}

// ----- Generator factory functions -----

fn gen_sequential_id(prefix: &str) -> GenFn {
    let prefix = prefix.to_string();
    Box::new(move |row_idx, _, _rng| format!("{}-{:06}", prefix, row_idx + 1))
}

fn gen_first_name() -> GenFn {
    Box::new(|_, _, rng| {
        let names = [
            "James",
            "Mary",
            "Robert",
            "Patricia",
            "John",
            "Jennifer",
            "Michael",
            "Linda",
            "David",
            "Elizabeth",
            "William",
            "Barbara",
            "Richard",
            "Susan",
            "Joseph",
            "Jessica",
            "Thomas",
            "Sarah",
            "Charles",
            "Karen",
            "Daniel",
            "Lisa",
            "Matthew",
            "Nancy",
            "Anthony",
            "Betty",
            "Mark",
            "Margaret",
            "Steven",
            "Sandra",
            "Andrew",
            "Ashley",
            "Paul",
            "Dorothy",
            "Joshua",
            "Kimberly",
            "Kenneth",
            "Emily",
            "Kevin",
            "Donna",
        ];
        names[rng.random_range(0..names.len())].to_string()
    })
}

fn gen_last_name() -> GenFn {
    Box::new(|_, _, rng| {
        let names = [
            "Smith",
            "Johnson",
            "Williams",
            "Brown",
            "Jones",
            "Garcia",
            "Miller",
            "Davis",
            "Rodriguez",
            "Martinez",
            "Hernandez",
            "Lopez",
            "Gonzalez",
            "Wilson",
            "Anderson",
            "Thomas",
            "Taylor",
            "Moore",
            "Jackson",
            "Martin",
            "Lee",
            "Perez",
            "Thompson",
            "White",
            "Harris",
            "Sanchez",
            "Clark",
            "Ramirez",
            "Lewis",
            "Robinson",
        ];
        names[rng.random_range(0..names.len())].to_string()
    })
}

fn gen_email() -> GenFn {
    Box::new(|row_idx, _, rng| {
        let domains = [
            "gmail.com",
            "yahoo.com",
            "outlook.com",
            "hotmail.com",
            "company.com",
            "mail.com",
        ];
        format!(
            "user{}@{}",
            row_idx + 1,
            domains[rng.random_range(0..domains.len())]
        )
    })
}

fn gen_phone() -> GenFn {
    Box::new(|_, _, rng| {
        format!(
            "({:03}) {:03}-{:04}",
            rng.random_range(200..999),
            rng.random_range(200..999),
            rng.random_range(1000..9999)
        )
    })
}

fn gen_city() -> GenFn {
    Box::new(|_, _, rng| {
        let cities = [
            "New York",
            "Los Angeles",
            "Chicago",
            "Houston",
            "Phoenix",
            "Philadelphia",
            "San Antonio",
            "San Diego",
            "Dallas",
            "San Jose",
            "Austin",
            "Jacksonville",
            "Fort Worth",
            "Columbus",
            "Charlotte",
            "Indianapolis",
            "San Francisco",
            "Seattle",
            "Denver",
            "Nashville",
            "Portland",
            "Miami",
            "Atlanta",
            "Boston",
            "Las Vegas",
        ];
        cities[rng.random_range(0..cities.len())].to_string()
    })
}

fn gen_state() -> GenFn {
    Box::new(|_, _, rng| {
        let states = [
            "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN",
            "IA", "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV",
            "NH", "NJ", "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN",
            "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
        ];
        states[rng.random_range(0..states.len())].to_string()
    })
}

fn gen_zip() -> GenFn {
    Box::new(|_, _, rng| format!("{:05}", rng.random_range(10000..99999)))
}

fn gen_country() -> GenFn {
    Box::new(|_, _, rng| {
        let countries = [
            "USA",
            "Canada",
            "UK",
            "Germany",
            "France",
            "Australia",
            "Japan",
            "Brazil",
            "India",
            "Mexico",
        ];
        countries[rng.random_range(0..countries.len())].to_string()
    })
}

fn gen_date(year_start: i32, year_end: i32) -> GenFn {
    Box::new(move |_, _, rng| {
        let year = rng.random_range(year_start..=year_end);
        let month = rng.random_range(1..=12);
        let day = rng.random_range(1..=28);
        format!("{:04}-{:02}-{:02}", year, month, day)
    })
}

fn gen_timestamp() -> GenFn {
    Box::new(|_, _, rng| {
        let year = rng.random_range(2023..=2025);
        let month = rng.random_range(1..=12);
        let day = rng.random_range(1..=28);
        let hour = rng.random_range(0..=23);
        let min = rng.random_range(0..=59);
        let sec = rng.random_range(0..=59);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year, month, day, hour, min, sec
        )
    })
}

fn gen_time() -> GenFn {
    Box::new(|_, _, rng| {
        let hour = rng.random_range(4..=20);
        let min = rng.random_range(0..=59);
        format!("{:02}:{:02}", hour, min)
    })
}

fn gen_int_range(min: i64, max: i64) -> GenFn {
    Box::new(move |_, _, rng| rng.random_range(min..=max).to_string())
}

fn gen_decimal(min: f64, max: f64, decimals: usize) -> GenFn {
    Box::new(move |_, _, rng| {
        let val: f64 = rng.random_range(min..=max);
        format!("{:.prec$}", val, prec = decimals)
    })
}

fn gen_pick(options: &[&str]) -> GenFn {
    let options: Vec<String> = options.iter().map(|s| s.to_string()).collect();
    Box::new(move |_, _, rng| {
        let val = &options[rng.random_range(0..options.len())];
        if val.contains(',') || val.contains(' ') {
            format!("\"{}\"", val)
        } else {
            val.clone()
        }
    })
}

fn gen_product() -> GenFn {
    Box::new(|_, _, rng| {
        let products = [
            "Laptop",
            "Phone",
            "Tablet",
            "Headphones",
            "Monitor",
            "Keyboard",
            "Mouse",
            "Camera",
            "Speaker",
            "Watch",
            "Printer",
            "Router",
            "Charger",
            "Cable",
            "Case",
            "Stand",
            "Adapter",
            "Battery",
            "Microphone",
            "Webcam",
        ];
        products[rng.random_range(0..products.len())].to_string()
    })
}

fn gen_campaign_name() -> GenFn {
    Box::new(|_, _, rng| {
        let adjectives = [
            "Spring", "Summer", "Fall", "Winter", "Holiday", "Flash", "Big", "Grand", "Special",
            "Premium",
        ];
        let nouns = [
            "Sale", "Promo", "Campaign", "Blitz", "Launch", "Drive", "Push", "Event", "Offer",
            "Deal",
        ];
        format!(
            "{} {}",
            adjectives[rng.random_range(0..adjectives.len())],
            nouns[rng.random_range(0..nouns.len())]
        )
    })
}

fn gen_computed_total(qty_col: usize, price_col: usize) -> GenFn {
    // This generates an independent value since we can't reference other columns easily
    let _ = (qty_col, price_col);
    Box::new(|_, _, rng| {
        let qty: f64 = rng.random_range(1.0..50.0);
        let price: f64 = rng.random_range(1.0..500.0);
        format!("{:.2}", qty * price)
    })
}

fn gen_random_column(col_seed: usize) -> GenFn {
    // Cycle through different random types based on column index
    match col_seed % 5 {
        0 => gen_int_range(1, 10000),
        1 => gen_decimal(0.0, 1000.0, 2),
        2 => gen_pick(&[
            "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
        ]),
        3 => gen_date(2020, 2025),
        _ => Box::new(|_, _, rng| {
            let len = rng.random_range(3..10);
            (0..len)
                .map(|_| rng.random_range(b'a'..=b'z') as char)
                .collect()
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: generate CSV into a String and return it.
    fn generate_to_string(rows: usize, cols: usize, gen_type: &str) -> String {
        let mut buf = Vec::new();
        generate_csv(&mut buf, rows, cols, gen_type).unwrap();
        String::from_utf8(buf).unwrap()
    }

    /// Helper: parse generated CSV into header + data rows.
    fn parse_csv(csv: &str) -> (Vec<String>, Vec<Vec<String>>) {
        let mut lines = csv.lines();
        let header: Vec<String> = lines
            .next()
            .unwrap()
            .split(',')
            .map(|s| s.to_string())
            .collect();
        let data: Vec<Vec<String>> = lines
            .filter(|l| !l.is_empty())
            .map(|l| l.split(',').map(|s| s.to_string()).collect())
            .collect();
        (header, data)
    }

    // ── validate_type ─────────────────────────────────────────

    #[test]
    fn test_validate_type_all_valid() {
        for t in &[
            "customer",
            "sales",
            "marketing",
            "weather",
            "scientific",
            "random",
        ] {
            assert!(validate_type(t).is_ok(), "expected '{}' to be valid", t);
        }
    }

    #[test]
    fn test_validate_type_invalid() {
        assert!(validate_type("invalid").is_err());
        assert!(validate_type("").is_err());
        assert!(validate_type("CUSTOMER").is_err()); // case-sensitive
    }

    // ── Row and column counts ─────────────────────────────────

    #[test]
    fn test_generate_row_count() {
        for &rows in &[0, 1, 5, 100] {
            let csv = generate_to_string(rows, 3, "random");
            let (_, data) = parse_csv(&csv);
            assert_eq!(data.len(), rows, "expected {} data rows", rows);
        }
    }

    #[test]
    fn test_generate_column_count() {
        for &cols in &[1, 5, 10, 20] {
            let csv = generate_to_string(3, cols, "random");
            let (header, data) = parse_csv(&csv);
            assert_eq!(header.len(), cols, "header should have {} columns", cols);
            for (i, row) in data.iter().enumerate() {
                assert!(
                    row.len() >= cols,
                    "row {} has {} cols, expected at least {}",
                    i,
                    row.len(),
                    cols
                );
            }
        }
    }

    // ── Typed headers ─────────────────────────────────────────

    #[test]
    fn test_customer_headers() {
        let csv = generate_to_string(1, 5, "customer");
        let (header, _) = parse_csv(&csv);
        assert_eq!(
            header,
            vec!["CustomerID", "FirstName", "LastName", "Email", "Phone"]
        );
    }

    #[test]
    fn test_sales_headers() {
        let csv = generate_to_string(1, 4, "sales");
        let (header, _) = parse_csv(&csv);
        assert_eq!(header, vec!["OrderID", "Date", "Product", "Category"]);
    }

    #[test]
    fn test_weather_headers() {
        let csv = generate_to_string(1, 3, "weather");
        let (header, _) = parse_csv(&csv);
        assert_eq!(header, vec!["Date", "City", "TempHigh"]);
    }

    #[test]
    fn test_marketing_headers() {
        let csv = generate_to_string(1, 3, "marketing");
        let (header, _) = parse_csv(&csv);
        assert_eq!(header, vec!["CampaignID", "CampaignName", "Channel"]);
    }

    #[test]
    fn test_scientific_headers() {
        let csv = generate_to_string(1, 3, "scientific");
        let (header, _) = parse_csv(&csv);
        assert_eq!(header, vec!["ExperimentID", "Timestamp", "SensorID"]);
    }

    #[test]
    fn test_random_headers() {
        let csv = generate_to_string(1, 4, "random");
        let (header, _) = parse_csv(&csv);
        assert_eq!(header, vec!["C1", "C2", "C3", "C4"]);
    }

    // ── Column overflow (more cols than typed headers) ────────

    #[test]
    fn test_extra_columns_get_generic_headers() {
        // customer has 15 typed headers; ask for 17
        let csv = generate_to_string(1, 17, "customer");
        let (header, _) = parse_csv(&csv);
        assert_eq!(header.len(), 17);
        assert_eq!(header[0], "CustomerID");
        assert_eq!(header[14], "LastPurchase"); // last typed
        assert_eq!(header[15], "C16"); // overflow
        assert_eq!(header[16], "C17");
    }

    // ── Fewer cols than typed headers ─────────────────────────

    #[test]
    fn test_fewer_columns_truncates_headers() {
        let csv = generate_to_string(1, 2, "customer");
        let (header, _) = parse_csv(&csv);
        assert_eq!(header, vec!["CustomerID", "FirstName"]);
    }

    // ── Sequential IDs ───────────────────────────────────────

    #[test]
    fn test_sequential_ids() {
        let csv = generate_to_string(3, 1, "customer");
        let (_, data) = parse_csv(&csv);
        assert_eq!(data[0][0], "CUST-000001");
        assert_eq!(data[1][0], "CUST-000002");
        assert_eq!(data[2][0], "CUST-000003");
    }

    // ── Output is valid CSV (parseable) ──────────────────────

    #[test]
    fn test_output_parseable_by_csv_crate() {
        for gen_type in VALID_TYPES {
            let mut buf = Vec::new();
            generate_csv(&mut buf, 50, 10, gen_type).unwrap();
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(buf.as_slice());
            let headers = reader.headers().unwrap();
            assert_eq!(headers.len(), 10, "type '{}' header count", gen_type);
            let records: Vec<_> = reader.records().collect();
            assert_eq!(records.len(), 50, "type '{}' row count", gen_type);
            for (i, rec) in records.iter().enumerate() {
                let rec = rec.as_ref().unwrap();
                assert!(
                    rec.len() >= 10,
                    "type '{}' row {} has {} fields",
                    gen_type,
                    i,
                    rec.len()
                );
            }
        }
    }

    // ── Zero rows produces header only ───────────────────────

    #[test]
    fn test_zero_rows_header_only() {
        let csv = generate_to_string(0, 5, "random");
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1); // header only
    }

    // ── Single column works ──────────────────────────────────

    #[test]
    fn test_single_column() {
        let csv = generate_to_string(5, 1, "random");
        let (header, data) = parse_csv(&csv);
        assert_eq!(header.len(), 1);
        assert_eq!(data.len(), 5);
    }
}
