use crate::error::SceneErr;
use std::io::Read;
use std::str::FromStr;

const SYMBOLS: [char; 7] = ['\n', ' ', '-', ':', '[', ',', ']'];

#[derive(Clone, Copy, Debug)]
pub struct SourceLocation<'a> {
    pub file_name: &'a str,
    pub line_num: u32,
    pub col_num: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Keywords {
    Brdf,
    Camera,
    Checkered,
    Compose,
    Diffusive,
    Image,
    Material,
    Materials,
    Name,
    Orthogonal,
    Perspective,
    Pigment,
    Plane,
    RotationX,
    RotationY,
    RotationZ,
    Scaling,
    Shapes,
    Specular,
    Sphere,
    Transformation,
    Transformations,
    Traslation,
    Type,
    Uniform,
}

enum Token<'a> {
    Keyword(SourceLocation<'a>, Keywords),
    Identifier(SourceLocation<'a>, String),
    String(SourceLocation<'a>, String),
    LiteralNumber(SourceLocation<'a>, f32),
    Symbol(SourceLocation<'a>, char),
    Stop(SourceLocation<'a>),
}

struct InputStream<'a, R: Read> {
    reader: R,
    location: SourceLocation<'a>,
    saved_ch: char,
    saved_location: SourceLocation<'a>,
    saved_token: Option<Token<'a>>,
}

impl<'a, R: Read> InputStream<'a, R> {
    pub fn new(reader: R, file_name: &'a str) -> Self {
        Self {
            reader,
            location: SourceLocation {
                file_name,
                line_num: 1,
                col_num: 1,
            },
            saved_ch: '\x00',
            saved_location: SourceLocation {
                file_name,
                line_num: 1,
                col_num: 1,
            },
            saved_token: None,
        }
    }

    fn update_pos(&mut self, ch: char) {
        if ch == '\n' {
            self.location.line_num += 1;
            self.location.col_num = 1;
        } else {
            self.location.col_num += 1
        }
    }

    fn read_char(&mut self) -> char {
        // Empty bytes buffer (only 1 byte).
        let mut ch = [0; 1];
        if self.saved_ch != '\x00' {
            ch[0] = self.saved_ch as u8;
            self.saved_ch = '\x00';
        } else {
            // Read a byte no matter if eof is reached.
            // A proper stop token will be raised (later).
            self.reader.read_exact(&mut ch).unwrap_or(());
        }
        self.saved_location = self.location;
        self.update_pos(ch[0] as char);
        ch[0] as char
    }

    fn unread_char(&mut self, ch: char) {
        self.saved_ch = ch;
        self.location = self.saved_location;
    }

    fn skip_comments(&mut self) {
        let mut ch = self.read_char();
        loop {
            // If a comment ignore until eol or eof.
            if ch == '#' {
                loop {
                    ch = self.read_char();
                    if ['\n', '\x00'].contains(&ch) {
                        break;
                    }
                }
                break;
            } else {
                // Roll back character.
                self.unread_char(ch);
                break;
            }
        }
    }

    fn parse_string(
        &mut self,
        token_location: SourceLocation<'a>,
        delimiter: char,
    ) -> Result<Token<'a>, SceneErr> {
        let mut ch;
        let mut token = String::from("");
        loop {
            ch = self.read_char();
            // If string delimiter `'` or `"` found, stop.
            if ch == delimiter {
                break;
            // If eof reached finding delimiter `'` or `"`, error.
            } else if ch == '\x00' {
                return Err(SceneErr::UnclosedString {
                    loc: token_location,
                    msg: format!("unclosed `{}`, untermineted string", delimiter),
                });
            } else {
                ();
            }
            token.push(ch);
        }
        Ok(Token::String(token_location, token))
    }

    fn parse_float(
        &mut self,
        first_char: char,
        token_location: SourceLocation<'a>,
    ) -> Result<Token<'a>, SceneErr> {
        let mut ch;
        let mut token = String::from("");
        token.push(first_char);
        loop {
            ch = self.read_char();
            // If `e` or `E` char found could be an float in scientific notation.
            if ch.to_ascii_lowercase() == 'e' {
                let ch_nx = self.read_char();
                // If the char follow exp char isn't a digit or a sign, unroll.
                if !(ch_nx.is_ascii_digit() || ['+', '-'].contains(&ch_nx)) {
                    // Unroll until exp char.
                    self.unread_char(ch_nx);
                    self.unread_char(ch);
                    break;
                } else {
                    // Roll both.
                    token.push(ch);
                    token.push(ch_nx);
                }
            } else {
                // If the char isn't a digit or a dot, unroll.
                if !(ch.is_ascii_digit() || ch == '.') {
                    self.unread_char(ch);
                    break;
                } else {
                    token.push(ch);
                }
            }
        }
        let value = f32::from_str(token.as_str()).map_err(|err| SceneErr::FloatParseFailure {
            loc: token_location,
            msg: format!("{} is an invalid floating point", token),
            src: err,
        })?;
        Ok(Token::LiteralNumber(token_location, value))
    }

    fn parse_keyword_or_identifier(
        &mut self,
        first_char: char,
        token_location: SourceLocation<'a>,
    ) -> Token<'a> {
        let mut ch;
        let mut token = String::from("");
        token.push(first_char);
        loop {
            ch = self.read_char();
            // `_` only accepted separator.
            if !(ch.is_ascii_alphanumeric() || ch == '_') {
                self.unread_char(ch);
                break;
            }
            token.push(ch);
        }
        match token.as_str() {
            "brdf" => Token::Keyword(token_location, Keywords::Brdf),
            "camera" => Token::Keyword(token_location, Keywords::Camera),
            "checkered" => Token::Keyword(token_location, Keywords::Checkered),
            "compose" => Token::Keyword(token_location, Keywords::Compose),
            "diffusive" => Token::Keyword(token_location, Keywords::Diffusive),
            "image" => Token::Keyword(token_location, Keywords::Image),
            "material" => Token::Keyword(token_location, Keywords::Material),
            "materials" => Token::Keyword(token_location, Keywords::Materials),
            "name" => Token::Keyword(token_location, Keywords::Name),
            "orthogonal" => Token::Keyword(token_location, Keywords::Orthogonal),
            "perspective" => Token::Keyword(token_location, Keywords::Perspective),
            "pigment" => Token::Keyword(token_location, Keywords::Pigment),
            "plane" => Token::Keyword(token_location, Keywords::Plane),
            "rotation_x" => Token::Keyword(token_location, Keywords::RotationX),
            "rotation_y" => Token::Keyword(token_location, Keywords::RotationY),
            "rotation_z" => Token::Keyword(token_location, Keywords::RotationZ),
            "scaling" => Token::Keyword(token_location, Keywords::Scaling),
            "shapes" => Token::Keyword(token_location, Keywords::Shapes),
            "specular" => Token::Keyword(token_location, Keywords::Specular),
            "sphere" => Token::Keyword(token_location, Keywords::Sphere),
            "transformation" => Token::Keyword(token_location, Keywords::Transformation),
            "transformations" => Token::Keyword(token_location, Keywords::Transformations),
            "traslation" => Token::Keyword(token_location, Keywords::Traslation),
            "type" => Token::Keyword(token_location, Keywords::Type),
            "uniform" => Token::Keyword(token_location, Keywords::Uniform),
            _ => Token::Identifier(token_location, token),
        }
    }

    fn read_token(&mut self) -> Result<Token, SceneErr> {
        self.skip_comments();
        let ch = self.read_char();
        // Save location where starting to parse the token.
        let token_location = self.location;
        if ch == '\x00' {
            Ok(Token::Stop(token_location))
        } else if SYMBOLS.contains(&ch) {
            let ch_nx = self.read_char();
            // Is negative number or special symbols?
            if ch == '-' && (ch_nx.is_ascii_digit() || ch_nx == '.') {
                self.unread_char(ch_nx);
                self.parse_float(ch, token_location)
            } else {
                self.unread_char(ch_nx);
                Ok(Token::Symbol(token_location, ch))
            }
        } else if ch.is_ascii_digit() || ch == '+' {
            self.parse_float(ch, token_location)
        } else if ch == '"' {
            self.parse_string(token_location, '"')
        } else if ch == '\'' {
            self.parse_string(token_location, '\'')
        } else if char::from(ch).is_ascii_alphabetic() || ch == '_' {
            Ok(self.parse_keyword_or_identifier(ch, token_location))
        } else {
            Err(SceneErr::InvalidCharacter {
                loc: self.location,
                msg: format!("{} invalid character", ch),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_unread() {
        let mut input = InputStream::new(Cursor::new("abc\nd\n#comment\nef"), "");

        assert_eq!(input.location.line_num, 1);
        assert_eq!(input.location.col_num, 1);

        assert_eq!(input.read_char(), 'a');
        assert_eq!(input.location.line_num, 1);
        assert_eq!(input.location.col_num, 2);

        input.unread_char('A');
        assert_eq!(input.location.line_num, 1);
        assert_eq!(input.location.col_num, 1);

        assert_eq!(input.read_char(), 'A');
        assert_eq!(input.location.line_num, 1);
        assert_eq!(input.location.col_num, 2);

        assert_eq!(input.read_char(), 'b');
        assert_eq!(input.location.line_num, 1);
        assert_eq!(input.location.col_num, 3);

        assert_eq!(input.read_char(), 'c');
        assert_eq!(input.location.line_num, 1);
        assert_eq!(input.location.col_num, 4);

        assert_eq!(input.read_char(), '\n');
        assert_eq!(input.location.line_num, 2);
        assert_eq!(input.location.col_num, 1);

        assert_eq!(input.read_char(), 'd');
        assert_eq!(input.location.line_num, 2);
        assert_eq!(input.location.col_num, 2);

        assert_eq!(input.read_char(), '\n');
        assert_eq!(input.location.line_num, 3);
        assert_eq!(input.location.col_num, 1);

        input.skip_comments();

        assert_eq!(input.read_char(), 'e');
        assert_eq!(input.location.line_num, 4);
        assert_eq!(input.location.col_num, 2);

        assert_eq!(input.read_char(), 'f');
        assert_eq!(input.location.line_num, 4);
        assert_eq!(input.location.col_num, 3);

        assert_eq!(input.read_char(), '\x00')
    }

    #[test]
    fn test_lexer() {
        let mut input = InputStream::new(
            Cursor::new(concat!(
                "# This is a comment\n",
                "transformations:\n",
                "  - name: camera_tr\n",
                "    compose:\n",
                "      - rotation_z: +1\n",
                "      - traslation: [-.3, 1E-02, 1E+1.5]\n",
                "\n",
                "@\n",
                "?\n",
                "materials:\n",
                "  - name: sphere_mt\n",
                "    type: diffusive\n",
                "    brdf:\n",
                "      image: \"path_to_image.pfm\"\n",
                "    pigment:\n",
                "      image: \'path_to_image.pfm\"\n",
            )),
            "",
        );

        assert!(
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::Transformations)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '-'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::Name)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Identifier(_loc, id)) if id == "camera_tr"));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::Compose)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '-'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::RotationZ)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::LiteralNumber(_loc, num)) if num == 1.0));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '-'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::Traslation)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '['));
        assert!(matches!(input.read_token(), Ok(Token::LiteralNumber(_loc, num)) if num == -0.3));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ','));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::LiteralNumber(_loc, num)) if num == 0.01));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ','));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(
            input.read_token(),
            Err(SceneErr::FloatParseFailure { loc, .. }) if loc.line_num==6 && loc.col_num==35));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ']'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(
            input.read_token(),
            Err(SceneErr::InvalidCharacter { loc, .. }) if loc.line_num==8 && loc.col_num==2));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(
            input.read_token(),
            Err(SceneErr::InvalidCharacter { loc, .. }) if loc.line_num==9 && loc.col_num==2));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        for _ in 1..35 {
            let _res = input.read_token();
        }
        assert!(
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::Image)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(
            matches!(input.read_token(), Ok(Token::String(_loc, st)) if st == "path_to_image.pfm")
        );
        for _ in 1..15 {
            let _res = input.read_token();
        }
        assert!(
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::Image)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(
            input.read_token(),
            Err(SceneErr::UnclosedString { loc, .. }) if loc.line_num==16 && loc.col_num==15));
        assert!(matches!(input.read_token(), Ok(Token::Stop(_loc))))
    }
}
