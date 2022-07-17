struct Parser<'a> {
    str: &'a str,
    idx: usize,
}

impl<'a> Parser<'a> {
    fn new(str: &'a str) -> Parser<'a> {
        Parser { str: str, idx: 0 }
    }

    fn eof(&self) -> bool {
        self.idx >= self.str.len()
    }

    fn peek(&self) -> char {
        self.str.chars().nth(self.idx).unwrap_or('\0')
    }

    fn pop(&mut self) -> char {
        let c = self.peek();
        self.idx += 1;
        c
    }

    fn consume(&mut self) {
        self.idx += 1
    }
}

pub fn preprocess_code(code: &str) -> String {
    let mut result = String::new();
    let mut parser = Parser::new(code);

    let mut comment_nesting = 0;
    let mut single_line = false;

    while !parser.eof() {
        let cur = parser.pop();
        if cur == '\r' {
            continue;
        }

        let nxt = parser.peek();
        if cur == '[' && nxt == '#' {
            comment_nesting += 1;
            parser.consume();
            continue;
        }
        if cur == '#' && nxt == ']' {
            assert!(comment_nesting > 0);
            comment_nesting -= 1;
            parser.consume();
            continue;
        }
        if cur == '#' && !single_line {
            comment_nesting += 1;
            single_line = true;
            continue;
        }
        if cur == '\n' && comment_nesting > 0 && single_line {
            single_line = false;
            comment_nesting -= 1;
        }

        if comment_nesting == 0 {
            result.push(cur);
        }
    }
    result
}
