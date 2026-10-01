//! A snapshot of what the parser makes of a fixed corpus: for every input, the IR it produces
//! (`Formula`'s Debug form: every instruction and every constant's exact bits) or the exact error,
//! hashed. Recorded in `snapshot.txt`; a change to either fails here, input by input.
//!
//! The corpus is the formulas found across the repository plus a deterministic stream of
//! generated ones that exercises every production of the grammar, and corrupted copies of them
//! for the errors. It exists to hold a refactor of the parser to "byte-neutral": first written for
//! the split that gave it a syntax tree (design/formula-textbook-editor.md §4.2).
//!
//! To re-record after a DELIBERATE change: `FRACTADYNE_BLESS_PARSE=1 cargo test -p fractadyne-core
//! parse_snapshot`, and say in the commit what changed and why.

use super::parse;

const FIXTURE: &str = include_str!("snapshot.txt");

/// Formulas from the repository: examples, tests, self-test cases, goldens, keypad snippets.
const KNOWN: &[&str] = &[
    "z^2 + c", "z = z^2 + c", "z = z*z + c", "sqr(z) + pixel", "z^3 + c", "z^4 + c", "z^5 + c",
    "t = sqr(z), z = t*t + c", "conj(z)^2 + c", "abs(z)^2 + c", "z = z^3 - p1*z + c",
    "t = sqr(z)\nz = t + p1*conj(t) + c", "z = (real(z) - flip(abs(imag(z))))^2 + c", "z = sin(z) + c",
    "z = exp(z) + c", "; Mandelbrot\nt = z*z ; square\nz = t\nz = z + c\n", "Z = Z*Z + C", "z = 5, z^2 + c",
    "25 + c", "z^2 + p1", "p2", "(0.5, -0.25)", "(-1.5, 2) * 2", "2^3 - 1", "pi", "e", "3*z", "z/4", "z/3",
    "-z^2", "2^3^2", "z^-1", "z^2.5", "z^0", "|z|", "cabs(z)", "abs(z)", "real(z) + imag(z)", "flip(z)",
    "conj(z)", "recip(z)", "2*3*4 + z", "z^2 + p3", "exp(z)", "log(z)", "sqrt(z)", "sin(z)", "cos(z)",
    "tan(z)", "sinh(z)", "cosh(z)", "tanh(z)", "ident(z)", "cotan(z)", "cotanh(z)", "z = z^2 +\n",
    "z = z^2 + c\nz = w + c", "z = sin(z", "z = fn1(z) + c", "if (|z| > 4)", "z = z < 2", "init: z = 0",
    "c = 3", "sin = 3", "z^c + c", "z^(z*0.5) + c", "z^2 + 0.1*log(z + 0.5) + c", "sqrt(z^4 + c)",
    "z^p1 + c", "z^2 + p1*z + c", "z = sin(z) + cos(z)*cos(z) + c", "z = 0.5*exp(z) - 0.5 + c",
    "z = 0.5*sinh(z) + c", "z = z^2*tanh(z) + c", "z = z^2 + c/(z + 2)", "z = z^2 + 0.3*log(z + 1) + c",
    "z = z^3 - p1*z +\nfn1(z)", "z = sin(z) + cos(z)*cos(z) + c", "z = z^4 - p1*z + c", "t = z\nz = t + c",
    "z + c ; comment", "t = z, z = t + c", "z^2 + 0.5", "z + (0.5, 0.5)", "sqrt(z) + c", "|z| + c",
    "z = (0.25, -0.1)*z + c", "z = z^2 + 0.25*z - (0, 1)*z/10 + c", "z = z^2 + c ; \u{221a} variant",
    "z = z + 1e-3", "z = z*2.5E2", "z = z^2 + 1e400", "z = (z, 1)", "z = |z", "z = )", "z = (", "= z",
    "z = z +* c", "z = z^^2 + c", "z = co", "z = zz", "tmp = sqr(z), Total = tmp*2\nz = total + c",
    "p1 = 2", "pixel = 1", "pi = 3", "z = 2z", "z = z..5", "z = .5 + z", "z = 3. + z", "",
];

/// A deterministic generator over the grammar.
struct Gen(u64);

impl Gen {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn pick(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn one<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[self.pick(xs.len())]
    }
    fn sp(&mut self) -> &'static str {
        ["", " ", " ", "  "][self.pick(4)]
    }
    fn number(&mut self) -> String {
        self.one(&["0", "1", "2", "3", "0.5", ".25", "3.", "10", "1e-3", "2.5E2", "1.5e-7", "64", "65", "0.1"]).to_string()
    }
    fn name(&mut self, vars: &[&str]) -> String {
        let mut names = vec!["z", "c", "pixel", "pi", "e", "p1", "p2", "p3", "p5", "Z", "PI", "u"];
        names.extend_from_slice(vars);
        self.one(&names).to_string()
    }
    fn expr(&mut self, depth: u32, vars: &[&str]) -> String {
        if depth == 0 || self.pick(5) == 0 {
            return if self.pick(2) == 0 { self.number() } else { self.name(vars) };
        }
        let d = depth - 1;
        match self.pick(12) {
            0..=3 => {
                let op = self.one(&["+", "-", "*", "/"]);
                let (a, b) = (self.expr(d, vars), self.expr(d, vars));
                let (s1, s2) = (self.sp(), self.sp());
                format!("{a}{s1}{op}{s2}{b}")
            }
            4 => {
                let f = self.one(&[
                    "exp", "log", "sqrt", "sin", "cos", "tan", "sinh", "cosh", "tanh", "sqr", "abs", "conj", "real",
                    "imag", "cabs", "flip", "recip", "ident", "cotan", "cotanh", "SIN",
                ]);
                format!("{f}({})", self.expr(d, vars))
            }
            5 => format!("({})", self.expr(d, vars)),
            6 => format!("|{}|", self.expr(d, vars)),
            7 => {
                let (a, b) = match self.pick(3) {
                    0 => (self.number(), self.number()),
                    1 => (format!("-{}", self.number()), self.number()),
                    _ => (self.expr(d, vars), self.number()),
                };
                format!("({a},{}{b})", self.sp())
            }
            8 => format!("-{}", self.expr(d, vars)),
            9 => format!("+{}", self.expr(d, vars)),
            _ => {
                let base = if self.pick(2) == 0 { self.name(vars) } else { format!("({})", self.expr(d, vars)) };
                let exp = match self.pick(6) {
                    0 => self.one(&["2", "3", "4", "0", "1", "64", "65"]).to_string(),
                    1 => format!("-{}", self.one(&["1", "2"])),
                    2 => self.one(&["2.5", "0.5", "1.5"]).to_string(),
                    3 => self.one(&["p1", "c", "z"]).to_string(),
                    4 => format!("({})", self.expr(d, vars)),
                    _ => format!("{}^{}", self.number(), self.number()),
                };
                format!("{base}^{exp}")
            }
        }
    }
    fn formula(&mut self) -> String {
        let mut out = String::new();
        let mut vars: Vec<&str> = Vec::new();
        let n = 1 + self.pick(3);
        for i in 0..n {
            if i > 0 {
                out.push_str(self.one(&["\n", ", ", ",", "\n\n"]));
            }
            match self.pick(4) {
                0 if i + 1 < n => {
                    let v = self.one(&["t", "w", "tmp", "a2"]);
                    out.push_str(&format!("{v} = {}", self.expr(3, &vars)));
                    vars.push(v);
                }
                1 => out.push_str(&self.expr(3, &vars)),
                _ => out.push_str(&format!("z = {}", self.expr(3, &vars))),
            }
            if self.pick(6) == 0 {
                out.push_str(" ; a comment (with parens)");
            }
        }
        out
    }
    /// A copy with one character deleted, inserted or changed — mostly errors.
    fn corrupt(&mut self, s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        if chars.is_empty() {
            return "(".into();
        }
        let at = self.pick(chars.len());
        let junk = ['(', ')', '|', ',', '=', '^', '*', '+', '-', 'q', '1', '\n', '<', ':'][self.pick(14)];
        let mut out = chars.clone();
        match self.pick(3) {
            0 => {
                out.remove(at);
            }
            1 => out.insert(at, junk),
            _ => out[at] = junk,
        }
        out.into_iter().collect()
    }
}

pub(super) fn corpus() -> Vec<String> {
    let mut out: Vec<String> = KNOWN.iter().map(|s| s.to_string()).collect();
    let mut g = Gen(0x5eed_f0f0_1234_abcd);
    for _ in 0..1500 {
        let f = g.formula();
        if g.pick(3) == 0 {
            out.push(g.corrupt(&f));
        }
        out.push(f);
    }
    out
}

fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}

fn fingerprint(src: &str) -> String {
    match parse(src) {
        Ok(f) => format!("{f:?}"),
        Err(e) => format!("ERR {e}"),
    }
}

#[test]
fn parse_snapshot() {
    let corpus = corpus();
    let lines: Vec<String> = corpus.iter().map(|s| format!("{:016x}", fnv(&fingerprint(s)))).collect();
    if std::env::var_os("FRACTADYNE_BLESS_PARSE").is_some() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/ir/parse/snapshot.txt");
        std::fs::write(path, lines.join("\n") + "\n").unwrap();
        return;
    }
    let want: Vec<&str> = FIXTURE.lines().collect();
    assert_eq!(want.len(), lines.len(), "the corpus changed size: re-record deliberately (see the module doc)");
    let ok = corpus.iter().filter(|s| parse(s).is_ok()).count();
    assert!(ok > 800 && corpus.len() - ok > 300, "the corpus lost its mix: {ok} parse of {}", corpus.len());
    let bad: Vec<String> = corpus
        .iter()
        .zip(lines.iter().zip(&want))
        .filter(|(_, (got, want))| got != *want)
        .take(10)
        .map(|(src, _)| format!("{src:?} -> {}", fingerprint(src).chars().take(160).collect::<String>()))
        .collect();
    assert!(bad.is_empty(), "the parser's output changed for:\n{}", bad.join("\n"));
}
