//! Scene parsing module.
//!
//! Provides `Scene` struct parsed from scene file (**yaml** formatted).
use crate::camera::{Camera, OrthogonalCamera, PerspectiveCamera};
use crate::cli::Cli;
use crate::color::{Color, BLACK, WHITE};
use crate::error::SceneErr;
use crate::hdrimage::HdrImage;
use crate::material::{
    CheckeredPigment, DiffuseBRDF, ImagePigment, Material, Pigment, SpecularBRDF, UniformPigment,
    BRDF,
};
use crate::shape::{Plane, RayIntersection, Sphere};
use crate::transformation::{
    rotation_x, rotation_y, rotation_z, scaling, translation, Transformation,
};
use crate::vector::{Vector, E1, E2, E3};
use crate::world::World;
use std::collections::BTreeMap;
use std::f32::consts::PI;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::str::FromStr;

/// Chars that must be considered special when parsed.
///
/// Because usually are separators or delimiters in the scene file.
const SYMBOLS: [char; 8] = ['\n', ' ', '-', ':', '[', ',', ']', '#'];

/// A specific position in a scene file.
#[derive(Clone, Copy, Debug)]
pub struct SourceLocation {
    /// Number of the line.
    pub line_num: u32,
    /// Number of the column.
    pub col_num: u32,
}

/// Enum for all the possible keywords of [`Token::Keyword`].
#[derive(Clone, Copy, Debug, PartialEq)]
enum Keywords {
    Camera,
    Checkered,
    Color,
    Colors,
    Compose,
    Diffuse,
    Distance,
    Image,
    Material,
    Materials,
    Name,
    Plane,
    Ratio,
    RotationX,
    RotationY,
    RotationZ,
    Scaling,
    Shapes,
    Specular,
    Sphere,
    Transformation,
    Transformations,
    Translation,
    Type,
    Uniform,
}

impl fmt::Display for Keywords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// Enum for all tokens recognized by the lexer.
#[derive(Debug, Clone)]
enum Token {
    Identifier(SourceLocation, String),
    Keyword(SourceLocation, Keywords),
    LiteralNumber(SourceLocation, f32),
    Stop(SourceLocation),
    String(SourceLocation, String),
    Symbol(SourceLocation, char),
}

/// Support macro when there is a token mismatch.
#[macro_export]
macro_rules! not_match {
    ($a:expr,$b:expr,$c:expr) => {
        Err(SceneErr::NotMatch {
            loc: $a,
            msg: format!(
                "found {:?} expected {}",
                format!("{}", $b),
                format!("{:?}", $c).to_lowercase().trim_matches('"')
            ),
        })
    };
}

/// Support macro to check what type of token mismatch occurs.
#[macro_export]
macro_rules! not_matches {
    ($a:expr,$c:expr) => {
        match $a {
            Token::Identifier(loc, id) => not_match!(loc, id, $c),
            Token::Keyword(loc, key) => not_match!(loc, key, $c),
            Token::LiteralNumber(loc, num) => not_match!(loc, num, $c),
            Token::Stop(loc) => not_match!(loc, '\x00', $c),
            Token::String(loc, st) => not_match!(loc, st, $c),
            Token::Symbol(loc, sym) => not_match!(loc, sym, $c),
        }
    };
}

/// A high-level wrapper around a stream, used to parse scene files (**yaml** formatted).
///
/// This class implements a wrapper around a stream,\
/// with the following additional capabilities:
///   * It tracks the line number and column number;
///   * It permits to "unread" characters and tokens;
///   * It tracks the number of spaces that build up a indent block.
#[derive(Clone)]
struct InputStream<R: Read> {
    /// A stream that implement [`Read`] trait.
    reader: R,
    /// A location pointer.
    location: SourceLocation,
    /// Last saved char.
    saved_ch: char,
    /// Last saved location.
    saved_location: SourceLocation,
    /// Last saved token.
    saved_token: Option<Token>,
    /// Spaces that build up an indent block.
    spaces: u32,
}

impl<R: Read> InputStream<R> {
    /// Create a new [`InputStream`] from a stream that implement [`Read`] trait.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            location: SourceLocation {
                line_num: 1,
                col_num: 1,
            },
            saved_ch: '\x00',
            saved_location: SourceLocation {
                line_num: 1,
                col_num: 1,
            },
            saved_token: None,
            spaces: 0,
        }
    }

    /// Update location after having read char from the stream.
    fn update_pos(&mut self, ch: char) {
        if ch == '\n' {
            self.location.line_num += 1;
            self.location.col_num = 1;
        } else {
            self.location.col_num += 1
        }
    }

    /// Read a new character from the stream.
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

    /// Push a character back to the stream.
    fn unread_char(&mut self, ch: char) {
        self.saved_ch = ch;
        self.location = self.saved_location;
    }

    /// If a comment character (for **yaml** is `#`) is found,\
    /// skip all the next ones until an end-of-line (`\n`) or end-of-file (`\x00`).
    fn skip_comment(&mut self) {
        let mut ch = self.read_char();
        // Strip all whitespaces before `#`.
        loop {
            if ch == ' ' {
                ch = self.read_char();
            } else {
                self.unread_char(ch);
                break;
            }
        }
        // Ignore inline comment.
        if ch == '#' {
            loop {
                ch = self.read_char();
                if ['\n', '\x00'].contains(&ch) {
                    self.unread_char(ch);
                    break;
                }
            }
        } else {
            self.unread_char(ch);
        }
    }

    /// Keep reading characters until a non-whitespace/non-comment character is found.
    fn skip_whitespaces_and_comments(&mut self) {
        let mut ch = self.read_char();
        loop {
            // If whitespaces or comment ignore.
            if [' ', '\n', '#'].contains(&ch) {
                if ch == '#' {
                    loop {
                        ch = self.read_char();
                        if ['\n', '\x00'].contains(&ch) {
                            break;
                        }
                    }
                }
                ch = self.read_char()
            } else {
                // Roll back character.
                self.unread_char(ch);
                break;
            }
        }
    }

    /// Count spaces that build up a particular indent block.\
    /// See [`parse_colors`](#method.parse_colors) for example of usage.
    fn count_spaces(&mut self) -> Result<(), SceneErr> {
        self.spaces = 1;
        let mut ch = self.read_char();
        loop {
            // Count multiple spaces occurrences.
            if ch == ' ' {
                self.spaces += 1;
                ch = self.read_char();
            } else {
                // Roll back character.
                self.unread_char(ch);
                break;
            }
        }
        Ok(())
    }

    /// Parse string token [`Token::String`].
    fn parse_string(
        &mut self,
        token_location: SourceLocation,
        delimiter: char,
    ) -> Result<Token, SceneErr> {
        let mut ch;
        let mut token = String::from("");
        loop {
            ch = self.read_char();
            // If string delimiter `'` or `"` found, stop.
            if ch == delimiter {
                break;
                // If eof or eol reached finding delimiter `'` or `"`, error.
            } else if ['\x00', '\n'].contains(&ch) {
                self.unread_char(ch);
                return Err(SceneErr::UnclosedString {
                    loc: self.location,
                    msg: format!("unclosed `{}` untermineted string", delimiter),
                });
            }
            token.push(ch);
        }
        Ok(Token::String(token_location, token))
    }

    /// Parse literal number (always as `f32`) token [`Token::LiteralNumber`].
    fn parse_float(
        &mut self,
        first_char: char,
        token_location: SourceLocation,
    ) -> Result<Token, SceneErr> {
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
            msg: format!("{:?} is an invalid floating-point number", token),
            src: err,
        })?;
        Ok(Token::LiteralNumber(token_location, value))
    }

    /// Parse a keyword token [`Token::Keyword`] or an identifier token [`Token::Identifier`].
    fn parse_keyword_or_identifier(
        &mut self,
        first_char: char,
        token_location: SourceLocation,
    ) -> Token {
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
            "camera" => Token::Keyword(token_location, Keywords::Camera),
            "checkered" => Token::Keyword(token_location, Keywords::Checkered),
            "color" => Token::Keyword(token_location, Keywords::Color),
            "colors" => Token::Keyword(token_location, Keywords::Colors),
            "compose" => Token::Keyword(token_location, Keywords::Compose),
            "diffuse" => Token::Keyword(token_location, Keywords::Diffuse),
            "distance" => Token::Keyword(token_location, Keywords::Distance),
            "image" => Token::Keyword(token_location, Keywords::Image),
            "material" => Token::Keyword(token_location, Keywords::Material),
            "materials" => Token::Keyword(token_location, Keywords::Materials),
            "name" => Token::Keyword(token_location, Keywords::Name),
            "plane" => Token::Keyword(token_location, Keywords::Plane),
            "ratio" => Token::Keyword(token_location, Keywords::Ratio),
            "rotationx" => Token::Keyword(token_location, Keywords::RotationX),
            "rotationy" => Token::Keyword(token_location, Keywords::RotationY),
            "rotationz" => Token::Keyword(token_location, Keywords::RotationZ),
            "scaling" => Token::Keyword(token_location, Keywords::Scaling),
            "shapes" => Token::Keyword(token_location, Keywords::Shapes),
            "specular" => Token::Keyword(token_location, Keywords::Specular),
            "sphere" => Token::Keyword(token_location, Keywords::Sphere),
            "transformation" => Token::Keyword(token_location, Keywords::Transformation),
            "transformations" => Token::Keyword(token_location, Keywords::Transformations),
            "translation" => Token::Keyword(token_location, Keywords::Translation),
            "type" => Token::Keyword(token_location, Keywords::Type),
            "uniform" => Token::Keyword(token_location, Keywords::Uniform),
            _ => Token::Identifier(token_location, token),
        }
    }

    /// Read a [`Token`] from the stream.
    ///
    /// If successful return a particular variant of [`Token`] enum wrapped inside [`Result`].\
    /// Otherwise return an error of type [`SceneErr::InvalidCharacter`].
    fn read_token(&mut self) -> Result<Token, SceneErr> {
        // If some saved token, use it.
        if self.saved_token.is_some() {
            let saved_token = self.saved_token.as_ref().unwrap().clone();
            self.saved_token = None;
            return Ok(saved_token);
        };
        // Save location where starting to parse token.
        let token_location = self.location;
        let ch = self.read_char();
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
        } else if ch.is_ascii_digit() || ['+', '.'].contains(&ch) {
            self.parse_float(ch, token_location)
        } else if ch == '"' {
            self.parse_string(token_location, '"')
        } else if ch == '\'' {
            self.parse_string(token_location, '\'')
        } else if ch.is_ascii_alphabetic() || ch == '_' {
            Ok(self.parse_keyword_or_identifier(ch, token_location))
        } else {
            //self.unread_char(ch);
            Err(SceneErr::InvalidCharacter {
                loc: token_location,
                msg: format!("{} invalid character", ch),
            })
        }
    }

    /// Make as if `token` were never read from stream.
    fn unread_token(&mut self, token: Token) {
        self.saved_token = Some(token)
    }

    /// Read a token from stream and check that it matches [`Token::Symbol`].\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn match_symbol(&mut self, symbol: char) -> Result<(), SceneErr> {
        let token = self.read_token()?;
        if matches!(token, Token::Symbol(_, sym) if sym==symbol) {
            Ok(())
        } else {
            not_matches!(token, symbol)
        }
    }

    /// Match an eof-of-inline or an inline comment, othewise return [`SceneErr::NotMatch`] error.
    fn match_eol_or_inline_comment(&mut self) -> Result<(), SceneErr> {
        let token = self.read_token()?;
        // Two possibility: eol or inline comment.
        if matches!(token, Token::Symbol(_, sym) if sym=='\n') {
            Ok(())
        } else if matches!(token, Token::Symbol(_, sym) if sym==' ') {
            self.skip_comment();
            self.match_symbol('\n')?;
            Ok(())
        } else {
            not_matches!(token, "inline comment or '\n'")
        }
    }

    /// Match whitespaces or an comments, othewise return [`SceneErr::NotMatch`] error.\
    /// Unread the `token`, and skip nothing only if `Token::Keyword` was parsed.
    fn match_whitespaces_and_comments(&mut self) -> Result<(), SceneErr> {
        let token = self.read_token()?;
        // If there is keyword or stop token (aka EOF), make it available for the next block.
        // So unread it.
        if matches!(token, Token::Keyword(_, _)) || matches!(token, Token::Stop(_)) {
            self.unread_token(token);
        // If there is comment symbol, unread the char '#',
        // and run `skip_whitespaces_and_comments`.
        } else if matches!(token, Token::Symbol(_, '#')) {
            self.unread_char('#');
            self.skip_whitespaces_and_comments();
        // Otherwise run `skip_whitespaces_and_comments`.
        } else {
            self.skip_whitespaces_and_comments();
        }
        Ok(())
    }

    /// Match the correct number of spaces for the current indent block.\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn match_spaces(&mut self, level: u32, nested: u32) -> Result<(), SceneErr> {
        // Match a particular number of spaces.
        // * `level` is intended for key alignment, incremented by 2 spaces.
        // * `nested` is intended for nested list alignment, incremented by `self.spaces`.
        for _ in 1..=(self.spaces + level * 2 + self.spaces * nested) {
            self.match_symbol(' ')?;
        }
        Ok(())
    }

    /// Read a token from stream and check that it matches [`Token::Keyword`] and\
    /// a particular `keywords` [`Keywords`].
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn match_keyword(&mut self, keyword: Keywords) -> Result<(), SceneErr> {
        // Match a particular keyword plus ':'.
        let token = self.read_token()?;
        match token {
            Token::Keyword(loc, key) => {
                if key == keyword {
                    self.match_symbol(':')
                } else {
                    not_match!(loc, key, keyword)
                }
            }
            _ => not_matches!(token, keyword),
        }
    }

    /// Read a token from stream and check that it matches [`Token::Keyword`] and\
    /// a particular range of `keywords` [`Keywords`].
    /// Return, wrapped inside a [`Result`], the keyword.\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn match_keywords(&mut self, keywords: &Vec<Keywords>) -> Result<Keywords, SceneErr> {
        // Match a potential vector of keywords plus ':'.
        let token = self.read_token()?;
        match token {
            Token::Keyword(loc, key) => {
                if keywords.contains(&key) {
                    self.match_symbol(':')?;
                    Ok(key)
                } else {
                    not_match!(loc, key, keywords)
                }
            }
            _ => not_matches!(token, keywords),
        }
    }

    /// Read a token from stream and check that it matches [`Token::Identifier`].\
    /// Return, wrapped inside a [`Result`], the identifier location and value.\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn match_identifier(&mut self) -> Result<(SourceLocation, String), SceneErr> {
        // Match a ' ' plus an identifier.
        self.match_symbol(' ')?;
        let token = self.read_token()?;
        match token {
            Token::Identifier(loc, id) => Ok((loc, id)),
            // If identifier is named as a keywords, no problem, use it as identifier.
            Token::Keyword(loc, key) => Ok((loc, format!("{:?}", key).to_lowercase())),
            _ => not_matches!(token, "identifier"),
        }
    }

    /// Read a token from stream and check that it matches [`Token::String`].\
    /// Return, wrapped inside a [`Result`], the string value and its location.\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn match_string(&mut self) -> Result<(SourceLocation, String), SceneErr> {
        let token = self.read_token()?;
        match token {
            Token::String(loc, st) => Ok((loc, st)),
            _ => not_matches!(token, "string"),
        }
    }

    /// Read a token from stream and check that it matches [`Token::LiteralNumber`].\
    /// Return, wrapped inside a [`Result`], the number value.\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn match_number(&mut self) -> Result<f32, SceneErr> {
        let token = self.read_token()?;
        match token {
            Token::LiteralNumber(_, num) => Ok(num),
            _ => not_matches!(token, "floating-point number"),
        }
    }

    /// Read a token from stream and check that it matches [`Token::LiteralNumber`] or
    /// a [`Token::Identifier`]\
    /// with a particular string instance, that if match means
    /// that [`f32`] number must be read from `cli`.\
    /// Return, wrapped inside a [`Result`], the number value.\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn match_number_cli(&mut self, cli: Cli) -> Result<f32, SceneErr> {
        let token = self.read_token()?;
        match token {
            Token::LiteralNumber(_, num) => Ok(num),
            Token::Identifier(loc, ref id) => {
                if id == "RATIO" {
                    Ok(cli.aspect_ratio)
                } else if id == "DISTANCE" {
                    Ok(1.0)
                } else {
                    Err(SceneErr::UndefinedIdentifier {
                        loc,
                        msg: format!(
                            "{:?} floating-point number not defined, available [DISTANCE, RATIO]",
                            id
                        ),
                    })
                }
            }
            _ => not_matches!(token, "floating-point number"),
        }
    }

    /// Parse a rgb color [`Color`] from stream combining previous match methods.\
    /// A color could be also read from a [`Token::Identifier`]
    /// if its string match a particular key of `var.colors` map.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_color(&mut self, var: &Var) -> Result<Color, SceneErr> {
        let token = self.read_token()?;
        match token {
            // Match a raw color rgb.
            Token::Symbol(_, '[') => {
                let r = self.match_number()?;
                self.match_symbol(',')?;
                self.match_symbol(' ')?;
                let g = self.match_number()?;
                self.match_symbol(',')?;
                self.match_symbol(' ')?;
                let b = self.match_number()?;
                self.match_symbol(']')?;
                Ok(Color::from((r, g, b)))
            }
            // Match color from variables `var`.
            Token::Identifier(loc, color) => {
                Ok(var
                    .colors
                    .get(&color)
                    .copied()
                    .ok_or(SceneErr::UndefinedIdentifier {
                        loc,
                        msg: format!("{:?} color not defined", color),
                    })?)
            }
            // Match color from variables `var`.
            Token::Keyword(loc, key) => Ok(var
                .colors
                .get(&format!("{:?}", key).to_lowercase())
                .copied()
                .ok_or(SceneErr::UndefinedIdentifier {
                    loc,
                    msg: format!("\"{:?}\" color not defined", key),
                })?),
            _ => not_matches!(token, "rgb color"),
        }
    }

    /// Parse an xyz vector [`Vector`] from stream combining previous match methods.\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn parse_vector(&mut self, var: &Var) -> Result<Vector, SceneErr> {
        let token = self.read_token()?;
        match token {
            // Match a raw vector xyz.
            Token::Symbol(_, '[') => {
                let x = self.match_number()?;
                self.match_symbol(',')?;
                self.match_symbol(' ')?;
                let y = self.match_number()?;
                self.match_symbol(',')?;
                self.match_symbol(' ')?;
                let z = self.match_number()?;
                self.match_symbol(']')?;
                Ok(Vector::from((x, y, z)))
            }
            // Match vector from variables `var`.
            Token::Identifier(loc, vector) => {
                Ok(var
                    .vectors
                    .get(&vector)
                    .copied()
                    .ok_or(SceneErr::UndefinedIdentifier {
                        loc,
                        msg: format!("{:?} vector not defined, available [E1, E2, E3]", vector),
                    })?)
            }
            _ => not_matches!(token, "xyz vector"),
        }
    }

    /// Parse a color from colors block combining [`parse_color`](#method.parse_color)
    /// and put it inside `var.colors` map.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_color_name(
        &mut self,
        colors: &mut BTreeMap<String, Color>,
        var: &Var,
    ) -> Result<(), SceneErr> {
        self.match_keyword(Keywords::Name)?;
        let (_, name) = self.match_identifier()?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // Match indent with colors block spaces + 1 level (2 spaces)
        self.match_spaces(1, 0)?;
        self.match_keyword(Keywords::Color)?;
        self.match_symbol(' ')?;
        colors.insert(name, self.parse_color(var)?);
        Ok(())
    }

    /// Parse colors inside colors block iterating [`parse_color_name`](#method.parse_color_name)
    /// until the block end.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_colors(&mut self, var: &Var) -> Result<BTreeMap<String, Color>, SceneErr> {
        let mut colors = BTreeMap::new();
        // The keyword `Keywords::Colors` is parsed inside `parse_scene`.
        // After 'colors:' can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // A minimum of one space indent is absolutely needed.
        self.match_symbol(' ')?;
        // Count spaces for colors block, used to parse indent.
        self.count_spaces()?;
        self.match_symbol('-')?;
        self.match_symbol(' ')?;
        self.parse_color_name(&mut colors, var)?;
        loop {
            // Can only be a eol or inline comment.
            self.match_eol_or_inline_comment()?;
            // Condition token: read a new color or not?
            let tk_nx = self.read_token()?;
            // If there is a space a new color can be parsed.
            // Otherwise stop with colors block.
            // No other suppositions are made! To reduce grammar complexity.
            // So for example no comment infra line of a block of colors.
            if matches!(tk_nx, Token::Symbol(_, sym) if sym==' ') {
                // Unread a space token to complete parse the correct
                // indent using `match_spaces`.
                self.unread_token(tk_nx);
                self.match_spaces(0, 0)?;
                self.match_symbol('-')?;
                self.match_symbol(' ')?;
                self.parse_color_name(&mut colors, var)?;
            } else {
                // Unread the condition token.
                self.unread_token(tk_nx);
                break;
            }
        }
        Ok(colors)
    }

    /// Parse a `pigment` [`Pigment`] from stream combining previous match and parse methods.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_pigment(&mut self, nested: u32, var: &Var) -> Result<Pigment, SceneErr> {
        // Match indent with materials block spaces + 1 level (2 spaces) +
        // + nested * (materials block spaces).
        self.match_spaces(1, nested)?;
        let pigment = self.match_keywords(&vec![
            Keywords::Uniform,
            Keywords::Checkered,
            Keywords::Image,
        ])?;
        self.match_symbol(' ')?;
        match pigment {
            Keywords::Uniform => Ok(Pigment::Uniform(UniformPigment {
                color: self.parse_color(var)?,
            })),
            Keywords::Image => {
                let (loc, pfm_file) = self.match_string()?;
                Ok(Pigment::Image(ImagePigment::new(
                    HdrImage::read_pfm_file(Path::new(&pfm_file)).map_err(|err| {
                        SceneErr::PfmFileReadFailure {
                            loc,
                            msg: format!("{:?} pfm file read failure", pfm_file),
                            src: err,
                        }
                    })?,
                )))
            }
            Keywords::Checkered => {
                self.match_symbol('[')?;
                let color1 = self.parse_color(var)?;
                self.match_symbol(',')?;
                self.match_symbol(' ')?;
                let color2 = self.parse_color(var)?;
                self.match_symbol(',')?;
                self.match_symbol(' ')?;
                let steps = self.match_number()? as u32;
                self.match_symbol(']')?;
                Ok(Pigment::Checkered(CheckeredPigment {
                    color1,
                    color2,
                    steps,
                }))
            }
            // This branch should never be triggered (a dummy error).
            _ => Err(SceneErr::UnexpectedMatch(String::from(
                "unexpected match (report it to devel)",
            ))),
        }
    }

    /// Parse a `brdf` [`BRDF`] from stream combining previous match methods.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_brdf(&mut self, var: &Var) -> Result<BRDF, SceneErr> {
        // Match indent with materials block spaces + 1 level (2 spaces).
        self.match_spaces(1, 0)?;
        let brdf = self.match_keywords(&vec![Keywords::Diffuse, Keywords::Specular])?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        match brdf {
            Keywords::Diffuse => Ok(BRDF::Diffuse(DiffuseBRDF {
                pigment: self.parse_pigment(1, var)?,
            })),
            Keywords::Specular => Ok(BRDF::Specular(SpecularBRDF {
                pigment: self.parse_pigment(1, var)?,
                threshold_angle_rad: PI / 1800.0,
            })),
            // This branch should never be triggered (a dummy error).
            _ => Err(SceneErr::UnexpectedMatch(String::from(
                "unexpected match (report it to devel)",
            ))),
        }
    }

    /// Parse a `material` [`Material`] inside materials block combining
    /// [`parse_pigment`](#method.parse_pigment) and [`parse_brdf`](#method.parse_brdf).\
    /// And put it inside `var.materials` map.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_material(
        &mut self,
        materials: &mut BTreeMap<String, Material>,
        var: &Var,
    ) -> Result<(), SceneErr> {
        self.match_keyword(Keywords::Name)?;
        let (_, name) = self.match_identifier()?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        let brdf = self.parse_brdf(var)?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        let emitted_radiance = self.parse_pigment(0, var)?;
        materials.insert(
            name,
            Material {
                brdf,
                emitted_radiance,
            },
        );
        Ok(())
    }

    /// Parse materials inside materials block iterating [`parse_material`](#method.parse_material)
    /// until the block end.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_materials(&mut self, var: &Var) -> Result<BTreeMap<String, Material>, SceneErr> {
        let mut materials = BTreeMap::new();
        // The keyword `Keywords::Materials` is parsed inside `parse_scene`.
        // After 'materials:' can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // A minimum of one space indent is absolutely needed.
        self.match_symbol(' ')?;
        // Count spaces for materials block, used to parse indent.
        self.count_spaces()?;
        self.match_symbol('-')?;
        self.match_symbol(' ')?;
        self.parse_material(&mut materials, var)?;
        loop {
            // Can only be a eol or inline comment.
            self.match_eol_or_inline_comment()?;
            // Condition token: read a new material or not?
            let tk_nx = self.read_token()?;
            // If there is a space a new material can be parsed.
            // Otherwise stop with materials block.
            // No other suppositions are made! To reduce grammar complexity.
            if matches!(tk_nx, Token::Symbol(_, sym) if sym==' ') {
                // Unread a space token to complete parse the correct
                // indent using `match_spaces`.
                self.unread_token(tk_nx);
                self.match_spaces(0, 0)?;
                self.match_symbol('-')?;
                self.match_symbol(' ')?;
                self.parse_material(&mut materials, var)?;
            } else {
                // Unread the condition token.
                self.unread_token(tk_nx);
                break;
            }
        }
        Ok(materials)
    }

    /// Parse a `transformation` [`Transformation`] from stream combining previous match methods.\
    /// Otherwise return a [`SceneErr::NotMatch`] error.
    fn parse_transformation(
        &mut self,
        transformations: &BTreeMap<String, Transformation>,
        var: &Var,
    ) -> Result<Transformation, SceneErr> {
        let transformation_tk = self.read_token()?;
        match transformation_tk {
            Token::Keyword(loc, key) => {
                let ch = self.read_char();
                // Match from transformation [`Token::Keywords`].
                if ch == ':' {
                    self.unread_char(':');
                    match key {
                        Keywords::RotationX => {
                            self.match_symbol(':')?;
                            self.match_symbol(' ')?;
                            Ok(rotation_x(f32::to_radians(self.match_number()?)))
                        }
                        Keywords::RotationY => {
                            self.match_symbol(':')?;
                            self.match_symbol(' ')?;
                            Ok(rotation_y(f32::to_radians(self.match_number()?)))
                        }
                        Keywords::RotationZ => {
                            self.match_symbol(':')?;
                            self.match_symbol(' ')?;
                            Ok(rotation_z(f32::to_radians(self.match_number()?)))
                        }
                        Keywords::Scaling => {
                            self.match_symbol(':')?;
                            self.match_symbol(' ')?;
                            Ok(scaling(self.parse_vector(var)?))
                        }
                        Keywords::Translation => {
                            self.match_symbol(':')?;
                            self.match_symbol(' ')?;
                            Ok(translation(self.parse_vector(var)?))
                        }
                        _ => not_matches!(
                            transformation_tk,
                            vec![
                                Keywords::RotationX,
                                Keywords::RotationY,
                                Keywords::RotationZ,
                                Keywords::Scaling,
                                Keywords::Translation
                            ]
                        ),
                    }
                // Match inside `transformations` [`BTreeMap`].
                } else {
                    self.unread_char(ch);
                    Ok(transformations
                        .get(&format!("{:?}", key).to_lowercase())
                        .copied()
                        .ok_or(SceneErr::UndefinedIdentifier {
                            loc,
                            msg: format!("\"{:?}\" transformation not defined", key),
                        })?)
                }
            }
            // Match inside `transformations` [`BTreeMap`].
            Token::Identifier(loc, id) => {
                transformations
                    .get(&id)
                    .copied()
                    .ok_or(SceneErr::UndefinedIdentifier {
                        loc,
                        msg: format!("{:?} transformation not defined", id),
                    })
            }
            _ => not_matches!(transformation_tk, "transformation"),
        }
    }

    /// Compose multiple `transformation` [`Transformation`] into one iterating over
    /// [`parse_transformation`](#method.parse_transformation).\
    /// And put it inside `var.transformations` map.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_composed_transformation(
        &mut self,
        transformations: &mut BTreeMap<String, Transformation>,
        var: &Var,
    ) -> Result<(), SceneErr> {
        // Init a identity translation to compose.
        let mut transformation = Transformation::default();
        self.match_keyword(Keywords::Name)?;
        let (_, name) = self.match_identifier()?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // Match indent with transformations block spaces + 1 level (2 spaces).
        self.match_spaces(1, 0)?;
        self.match_keyword(Keywords::Compose)?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // Match indent with transformations block spaces + 1 level (2 spaces) +
        // + 1 * (transformations block spaces).
        self.match_spaces(1, 1)?;
        self.match_symbol('-')?;
        self.match_symbol(' ')?;
        transformation = transformation * self.parse_transformation(transformations, var)?;
        loop {
            // Can only be a eol or inline comment.
            self.match_eol_or_inline_comment()?;
            // Condition token: continue to read or not?
            let tk_nx = self.read_token()?;
            if matches!(tk_nx, Token::Symbol(_, sym) if sym==' ') {
                // Unread a space token to complete parse the correct
                // indent using `match_spaces`.
                self.unread_token(tk_nx);
                self.match_spaces(0, 0)?;
                // Condition token (again): continue to compose current
                // transformation or not?
                let tk_nx_nx = self.read_token()?;
                match tk_nx_nx {
                    // If there is a space (again) continue to compose.
                    Token::Symbol(_, ' ') => {
                        // Unread a space token to complete parse the correct
                        // indent using `match_spaces`.
                        self.unread_token(tk_nx_nx);
                        // Match indent with transformations block spaces + 1 level (2 spaces).
                        self.match_spaces(1, 0)?;
                        self.match_symbol('-')?;
                        self.match_symbol(' ')?;
                        transformation =
                            transformation * self.parse_transformation(transformations, var)?;
                        Ok(())
                    }
                    // Otherwise stop with compose transformation block.
                    // If there is '-' no more composing, this is a new transformation.
                    Token::Symbol(_, '-') => {
                        // What! This branch has diffent return type `()` so not `Result`,
                        // but `break` keyword make this match valid as the others.
                        // Magic rustc.
                        self.unread_token(tk_nx_nx);
                        break;
                    }
                    // No other suppositions are made! To reduce grammar complexity.
                    _ => not_matches!(tk_nx_nx, "[' ', '-']"),
                }?;
            } else {
                // Unread the condition token.
                self.unread_token(tk_nx);
                break;
            }
        }
        transformations.insert(name, transformation);
        Ok(())
    }

    /// Parse transformations inside transformations block iterating
    /// [`parse_composed_transformation`](#method.parse_composed_transformation)
    /// until the block end.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_transformations(
        &mut self,
        var: &Var,
    ) -> Result<BTreeMap<String, Transformation>, SceneErr> {
        let mut transformations = BTreeMap::new();
        // The keyword `Keywords::Transformations` is parsed inside `parse_scene`.
        // After 'transformations:' can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // A minimum of one space indent is absolutely needed.
        self.match_symbol(' ')?;
        // Count spaces for transformations block, used to parse indent.
        self.count_spaces()?;
        self.match_symbol('-')?;
        self.match_symbol(' ')?;
        self.parse_composed_transformation(&mut transformations, var)?;
        loop {
            // Condition token: read a new transformation or not?
            let tk_nx = self.read_token()?;
            // If there is '-' read new transformation.
            // Otherwise stop with materials block.
            // No other suppositions are made! To reduce grammar complexity.
            if matches!(tk_nx, Token::Symbol(_, sym) if sym=='-') {
                self.match_symbol(' ')?;
                self.parse_composed_transformation(&mut transformations, var)?;
            } else {
                // Unread condition token.
                self.unread_token(tk_nx);
                break;
            }
        }
        Ok(transformations)
    }

    /// Parse shape inside shapes block using `var.materials` and `var.transformations`.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_shape(&mut self, var: &Var) -> Result<Box<dyn RayIntersection>, SceneErr> {
        let shape = self.match_keywords(&vec![Keywords::Plane, Keywords::Sphere])?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // Match indent with shapes block spaces + 1 level (2 spaces).
        self.match_spaces(1, 0)?;
        self.match_keyword(Keywords::Material)?;
        let (loc, material_id) = self.match_identifier()?;
        // Match `material_id` from variables `var`.
        let material =
            var.materials
                .get(&material_id)
                .cloned()
                .ok_or(SceneErr::UndefinedIdentifier {
                    loc,
                    msg: format!("{:?} material not defined", material_id),
                })?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // Match indent with shapes block spaces + 1 level (2 spaces).
        self.match_spaces(1, 0)?;
        self.match_keyword(Keywords::Transformation)?;
        let (loc, transformation_id) = self.match_identifier()?;
        // Match `transformation_id` from variables `var`.
        let transformation = var.transformations.get(&transformation_id).copied().ok_or(
            SceneErr::UndefinedIdentifier {
                loc,
                msg: format!("{:?} transformation not defined", transformation_id),
            },
        )?;
        match shape {
            Keywords::Plane => Ok(Box::new(Plane::new(transformation, material))),
            Keywords::Sphere => Ok(Box::new(Sphere::new(transformation, material))),
            // This branch should never be triggered (a dummy error).
            _ => Err(SceneErr::UnexpectedMatch(String::from(
                "unexpected match (report it to devel)",
            ))),
        }
    }

    /// Parse shapes inside shapes block iterating
    /// [`parse_shape`](#method.parse_shape) until the block end.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_shapes(&mut self, var: &Var) -> Result<World, SceneErr> {
        // Init an empty world object.
        let mut shapes = World::default();
        // The keyword `Keywords::Shapes` is parsed inside `parse_scene`.
        // After 'shapes:' can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // A minimum of one space indent is absolutely needed.
        self.match_symbol(' ')?;
        // Count spaces for shapes block, used to parse indent.
        self.count_spaces()?;
        self.match_symbol('-')?;
        self.match_symbol(' ')?;
        shapes.add(self.parse_shape(var)?);
        loop {
            // Can only be a eol or inline comment.
            self.match_eol_or_inline_comment()?;
            // Condition token: read a new shape or not?
            let tk_nx = self.read_token()?;
            // If there is a space a new shape can be parsed.
            // Otherwise stop with shapes block.
            // No other suppositions are made! To reduce grammar complexity.
            // So for example no comment infra line of a block of shapes.
            if matches!(tk_nx, Token::Symbol(_, sym) if sym==' ') {
                // Unread a space token to complete parse the correct
                // indent using `match_spaces`.
                self.unread_token(tk_nx);
                self.match_spaces(0, 0)?;
                self.match_symbol('-')?;
                self.match_symbol(' ')?;
                shapes.add(self.parse_shape(var)?);
            } else {
                // Unread condition token.
                self.unread_token(tk_nx);
                break;
            }
        }
        Ok(shapes)
    }

    /// Parse camera inside camera block using `var.materials` and `var.transformations`,\
    /// and optionally for particular identifiers read standard values from `cli`.\
    /// Otherwise return a variant of [`SceneErr`] error.
    fn parse_camera(&mut self, var: &Var, cli: Cli) -> Result<Camera, SceneErr> {
        // The keyword `Keywords::Camera` is parsed inside `parse_scene`.
        // After 'camera:' can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // A minimum of one space indent is absolutely needed.
        self.match_symbol(' ')?;
        // Count spaces for camera block, used to parse indent.
        self.count_spaces()?;
        self.match_keyword(Keywords::Type)?;
        self.match_symbol(' ')?;
        // Fail fast if invalid camera type parsed.
        let (loc, camera) = self.match_string()?;
        if !(camera == "orthogonal" || camera == "perspective") {
            return Err(SceneErr::InvalidCamera {
                loc,
                msg: format!(
                    "found {:?} camera expected [\"orthogonal\", \"perspective\"]",
                    camera
                ),
            });
        }
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // Match indent with camera block spaces.
        self.match_spaces(0, 0)?;
        self.match_keyword(Keywords::Ratio)?;
        self.match_symbol(' ')?;
        let ratio = self.match_number_cli(cli)?;
        // Can only be a eol or inline comment.
        self.match_eol_or_inline_comment()?;
        // Init a default distance.
        let mut distance = 1.0;
        // Override it if the camera is perspective,
        // otherwise will remain unused.
        if camera == "perspective" {
            // Match indent with camera block spaces.
            self.match_spaces(0, 0)?;
            self.match_keyword(Keywords::Distance)?;
            self.match_symbol(' ')?;
            distance = self.match_number_cli(cli)?;
            // Can only be a eol or inline comment.
            self.match_eol_or_inline_comment()?;
        }
        // Match indent with camera block spaces.
        self.match_spaces(0, 0)?;
        self.match_keyword(Keywords::Transformation)?;
        let (loc, transformation_id) = self.match_identifier()?;
        // Match `transformation_id` from variables `var`.
        let transformation = var.transformations.get(&transformation_id).copied().ok_or(
            SceneErr::UndefinedIdentifier {
                loc,
                msg: format!("{:?} transformation not defined", transformation_id),
            },
        )? * rotation_z(f32::to_radians(cli.angle_deg));
        match camera.as_str() {
            "orthogonal" => Ok(Camera::Orthogonal(OrthogonalCamera::new(
                ratio,
                transformation,
            ))),
            "perspective" => Ok(Camera::Perspective(PerspectiveCamera::new(
                distance,
                ratio,
                transformation,
            ))),
            // This branch should never be triggered (a dummy error).
            _ => Err(SceneErr::UnexpectedMatch(String::from(
                "unexpected match (report it to devel)",
            ))),
        }
    }

    /// Parse a scene in all its entirety.
    ///
    /// Blocks that must exist:
    ///  * camera;
    ///  * materials;
    ///  * shapes.
    ///
    /// Optionals:
    ///  * colors;
    ///  * transformations.
    ///
    /// Blocks can be separated by multiple break line.
    ///
    /// When a camera and world (list of shapes) are parsed stop scene parsing.
    fn parse_scene(&mut self, cli: Cli) -> Result<Scene, SceneErr> {
        let mut block;
        let mut var = Var::default();
        let mut scene = Scene::default();
        let mut blocks = vec![
            Keywords::Camera,
            Keywords::Colors,
            Keywords::Materials,
            Keywords::Shapes,
            Keywords::Transformations,
        ];
        // Loop over expected blocks until `Camera` and `World` are created.
        // Or until eof is reached.
        loop {
            if !(scene.camera.is_some() && scene.shapes.is_some()) {
                // Try to ignore whitespaces and comments infra-blocks.
                self.match_whitespaces_and_comments()?;
                block = self.match_keywords(&blocks)?;
                match block {
                    // Build a `Camera` in `scene` using `var`.
                    // And remove it from `blocks`, because was found.
                    Keywords::Camera => {
                        scene.camera = Some(self.parse_camera(&var, cli)?);
                        blocks.remove(blocks.iter().position(|&k| k == Keywords::Camera).unwrap());
                    }
                    // Update colors in `var` if colors block is found.
                    // And remove it from `blocks`, because was found.
                    Keywords::Colors => {
                        var.colors.append(&mut self.parse_colors(&var)?);
                        blocks.remove(blocks.iter().position(|&k| k == Keywords::Colors).unwrap());
                    }
                    // Update materials in `var` if materials block is found.
                    // And remove it from `blocks`, because was found.
                    Keywords::Materials => {
                        var.materials.append(&mut self.parse_materials(&var)?);
                        blocks.remove(
                            blocks
                                .iter()
                                .position(|&k| k == Keywords::Materials)
                                .unwrap(),
                        );
                    }
                    // Build a `World` in `scene` using `var`.
                    // And remove it from `blocks`, because was found.
                    Keywords::Shapes => {
                        scene.shapes = Some(self.parse_shapes(&var)?);
                        blocks.remove(blocks.iter().position(|&k| k == Keywords::Shapes).unwrap());
                    }
                    // Update transformations in `var` if transformations block is found.
                    // And remove it from `blocks`, because was found.
                    Keywords::Transformations => {
                        var.transformations
                            .append(&mut self.parse_transformations(&var)?);
                        blocks.remove(
                            blocks
                                .iter()
                                .position(|&k| k == Keywords::Transformations)
                                .unwrap(),
                        );
                    }
                    // This branch should never be triggered (do nothing).
                    _ => (),
                };
            } else {
                break;
            }
        }
        Ok(scene)
    }
}

/// Variables object, useful to store when parsing.
///
/// Struct that wrap different [`BTreeMap`] for each of scene blocks.
/// Ready to be used when parsing a [`Token::Identifier`]
/// with a key that exist for the particular block.
#[derive(Clone)]
struct Var {
    /// Map of colors.
    colors: BTreeMap<String, Color>,
    /// Map of materials.
    materials: BTreeMap<String, Material>,
    /// Map of transformations.
    transformations: BTreeMap<String, Transformation>,
    /// Map of vectors.
    vectors: BTreeMap<String, Vector>,
}

impl Default for Var {
    /// Initialize a variables object with some useful predefined keys.\
    /// E.g. "BLACK" and "WHITE" keys for [`BLACK`] and [`WHITE`] colors.
    fn default() -> Self {
        let mut colors = BTreeMap::new();
        colors.insert(String::from("BLACK"), BLACK);
        colors.insert(String::from("WHITE"), WHITE);
        let materials = BTreeMap::new();
        let mut transformations = BTreeMap::new();
        transformations.insert(String::from("IDENTITY"), Transformation::default());
        let mut vectors = BTreeMap::new();
        vectors.insert(String::from("E1"), E1);
        vectors.insert(String::from("E2"), E2);
        vectors.insert(String::from("E3"), E3);
        Self {
            colors,
            materials,
            transformations,
            vectors,
        }
    }
}

/// Scene to render.
///
/// Usually parsed from a scene file.
#[derive(Debug, Default)]
pub struct Scene {
    pub camera: Option<Camera>,
    pub shapes: Option<World>,
}

impl Scene {
    /// Build up scene from a scene file (**yaml** formatted).
    ///
    /// Wrapper around [`parse_scene`](../scene/struct.InputStream.html#method.parse_scene)
    /// method of [`InputStream`].
    pub fn read_scene_file(path: &Path, cli: Cli) -> Result<Self, SceneErr> {
        let file = File::open(path).map_err(SceneErr::SceneFileReadFailure)?;
        let reader = BufReader::new(file);
        let mut input = InputStream::new(reader);
        input.parse_scene(cli)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{BufWriter, Cursor, Write};

    #[test]
    fn test_read_unread() {
        let mut input = InputStream::new(Cursor::new("abc\nd \n  #comment\nef"));

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

        input.skip_whitespaces_and_comments();

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
        let mut input = InputStream::new(Cursor::new(concat!(
            "\n",
            "\n",
            " # This is a comment\n",
            "transformations:\n",
            "  - name: camera_tr\n",
            "    compose:\n",
            "      - rotationz: +1\n",
            "      - rotationy: .5\n",
            "      - translation: [-.3, 1E-02, 1E+1.5]\n",
            "\n",
            "\n",
            "?\n",
            "materials:\n",
            "  - name: sphere_mt\n",
            "    diffuse:\n",
            "      image: \"path_to_image.pfm\"\n",
            "    image: \'path_to_image.pfm\"\n",
        )));

        input.skip_whitespaces_and_comments();
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
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::RotationY)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        assert!(matches!(input.read_token(), Ok(Token::LiteralNumber(_loc, num)) if num == 0.5));
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
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::Translation)
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
        Err(SceneErr::FloatParseFailure { loc, .. }) if loc.line_num==9 && loc.col_num==35));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ']'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(matches!(
        input.read_token(),
        Err(SceneErr::InvalidCharacter { loc, .. }) if loc.line_num==12 && loc.col_num==1));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        for _ in 1..=25 {
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
        for _ in 1..=5 {
            let _res = input.read_token();
        }
        assert!(
            matches!(input.read_token(), Ok(Token::Keyword(_loc, key)) if key == Keywords::Image)
        );
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ':'));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == ' '));
        //println!("{:?}", input.read_token());
        //assert!(false);
        assert!(matches!(
        input.read_token(),
        Err(SceneErr::UnclosedString { loc, .. }) if loc.line_num==17 && loc.col_num==31));
        assert!(matches!(input.read_token(), Ok(Token::Symbol(_loc, sym)) if sym == '\n'));
        assert!(
            matches!(input.read_token(), Ok(Token::Stop(loc)) if loc.line_num==18 && loc.col_num==1)
        )
    }

    #[test]
    fn test_camera_parser() {
        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "camera:\n",
            "   type: 'perspective'\n",
            "   ratio: 0.5\n",
            "   distance: DISTANCE\n",
            "   transformation: IDENTITY\n",
        )));
        let mut var = Var::default();
        let cli = Cli {
            aspect_ratio: 0.5,
            angle_deg: 0.0,
        };
        var.transformations
            .insert(String::from("camera"), Transformation::default());

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Camera).is_ok());
        assert!(
            matches!(input.parse_camera(&var, cli), Ok(Camera::Perspective(cam)) if cam==PerspectiveCamera::new(1.0, 0.5, Transformation::default()))
        );

        let mut input = InputStream::new(Cursor::new(concat!(
            "camera:\n",
            "  type: \"orthogonal\"    # This is an inline comment\n",
            "  ratio: RATIO\n",
            "  transformation: camera\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Camera).is_ok());
        assert!(
            matches!(input.parse_camera(&var, cli), Ok(Camera::Orthogonal(cam)) if cam==OrthogonalCamera::new(0.5, Transformation::default()))
        );

        let mut input = InputStream::new(Cursor::new(concat!(
            "\n",
            "\n  # This is a double spaced comment",
            "\n",
            "camera:\n",
            "  type: 'mycamera'\n",
            "  ratio: 0.5\n",
            "  distance: 1.0\n",
            "  transformation: camera\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Camera).is_ok());
        assert!(matches!(
            input.parse_camera(&var, cli),
            Err(SceneErr::InvalidCamera { loc, .. }) if loc.line_num==5 && loc.col_num==9
        ));

        let mut input = InputStream::new(Cursor::new(concat!(
            "\n  ",
            "camera:\n",
            "  type: 'perspective'\n",
            "  ratio: 0.5\n",
            "  distance: 1.0\n",
            "  transformation: camera2\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Camera).is_ok());
        assert!(matches!(
            input.parse_camera(&var, cli),
            Err(SceneErr::UndefinedIdentifier { loc, .. }) if loc.line_num==6 && loc.col_num==19
        ));

        let mut input = InputStream::new(Cursor::new(concat!(
            "camera:\n",
            "  type: 'perspective'\n",
            " ratio: 0.5\n",
            "  distance: 1.0\n",
            "  transformation: camera2\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Camera).is_ok());
        assert!(matches!(
            input.parse_camera(&var, cli),
            Err(SceneErr::NotMatch { loc, .. }) if loc.line_num==3 && loc.col_num==2
        ))
    }

    #[test]
    fn test_colors_parser() {
        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "colors:\n",
            "   - name: red\n",
            "     color: [1.0, 0., 0]\n",
            "   - name: green\n",
            "     color: [0.0, 1., 0]\n",
            "   - name: blue\n",
            "     color: [0.0, 0., 1]\n",
        )));
        let var: Var = Var::default();
        let red = Color::from((1.0, 0.0, 0.0));
        let green = Color::from((0.0, 1.0, 0.0));
        let blue = Color::from((0.0, 0.0, 1.0));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Colors).is_ok());
        let colors = input.parse_colors(&var);
        assert!(colors.is_ok());
        assert!(matches!(colors.as_ref().unwrap().get("red"), Some(r) if *r==red));
        assert!(matches!(colors.as_ref().unwrap().get("green"), Some(g) if *g==green));
        assert!(matches!(colors.as_ref().unwrap().get("blue"), Some(b) if *b==blue));

        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "colors:\n",
            "  - name: red\n",
            "    color: [1.0, 0., 0]\n",
            "  - name: green\n",
            "    colors: [0.0, 1., 0]\n",
            "  - name: blue\n",
            "    color: [0.0, 0., 1]\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Colors).is_ok());
        assert!(matches!(
            input.parse_colors(&var),
            Err(SceneErr::NotMatch { loc, .. }) if loc.line_num==6 && loc.col_num==5
        ));

        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "colors:\n",
            "       - name: red\n",
            "         color: [1.0, 0., 0]\n",
            "       - name: green\n",
            "         colors: [0.0, 1., 0]\n",
            "       - name: blue\n",
            "         color: [0.0, 0., 1]\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Colors).is_ok())
    }

    #[test]
    fn test_materials_parser() {
        let mut input = InputStream::new(Cursor::new(concat!(
            "materials:\n",
            "   - name: sky\n",
            "     specular:\n",
            "        uniform: [1.2, 0.9, 3.7]\n",
            "     uniform: plane # This is an inline comment\n",
            "   - name: ground\n",
            "     diffuse:\n",
            "        checkered: [BLACK, WHITE, 7.]\n",
            "     uniform: [2.1, 9.0, 7.3]\n",
        )));
        let mut var: Var = Var::default();
        let sky_brdf_uniform = Color::from((1.2, 0.9, 3.7));
        let sky_radiance_uniform = Color::from((2.1, 9.0, 7.3));
        let ground_brdf_checkered_c1 = BLACK;
        let ground_brdf_checkered_c2 = WHITE;
        let ground_brdf_checkered_steps = 7;
        let ground_radiance_uniform = sky_radiance_uniform;
        var.colors
            .insert(String::from("plane"), sky_radiance_uniform);

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Materials).is_ok());
        let materials = input.parse_materials(&var);
        assert!(materials.is_ok());
        assert!(
            matches!(materials.as_ref().unwrap().get("sky"), Some(sky) if (matches!(&sky.brdf, BRDF::Specular(sp) if matches!(sp.pigment, Pigment::Uniform(pg) if pg.color==sky_brdf_uniform))) && (matches!(&sky.emitted_radiance, Pigment::Uniform(pg) if pg.color==sky_radiance_uniform))
            )
        );
        assert!(
            matches!(materials.as_ref().unwrap().get("ground"), Some(ground) if (matches!(&ground.brdf, BRDF::Diffuse(df) if matches!(df.pigment, Pigment::Checkered(pg) if pg.color1==ground_brdf_checkered_c1 && pg.color2==ground_brdf_checkered_c2 && pg.steps==ground_brdf_checkered_steps))) && (matches!(&ground.emitted_radiance, Pigment::Uniform(pg) if pg.color==ground_radiance_uniform))
            )
        );

        let mut input = InputStream::new(Cursor::new(concat!(
            "materials:\n",
            "  - name: sky\n",
            "    specular:\n",
            "      image: 'not_found.pfm'\n",
            "    uniform: [2.1, 9.0, 7.3]\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Materials).is_ok());
        assert!(matches!(
        input.parse_materials(&var),
        Err(SceneErr::PfmFileReadFailure { loc, .. }) if loc.line_num==4 && loc.col_num==14
        ));

        let mut input = InputStream::new(Cursor::new(concat!(
            "materials:\n",
            "  - name: sky\n",
            "    reflex:\n",
            "      image: 'not_found.pfm'\n",
            "    uniform: [2.1, 9.0, 7.3]\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Materials).is_ok());
        assert!(matches!(
            input.parse_materials(&var),
            Err(SceneErr::NotMatch { loc, .. }) if loc.line_num==3 && loc.col_num==5
        ));

        let mut input = InputStream::new(Cursor::new(concat!(
            "materials:\n",
            "  - name: sky\n",
            "    specular:\n",
            "      uniform: [1.2, 1.3, 1.4]\n",
            "    not_uniform: [2.1, 9.0, 7.3]\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Materials).is_ok());
        assert!(matches!(
            input.parse_materials(&var),
            Err(SceneErr::NotMatch { loc, .. }) if loc.line_num==5 && loc.col_num==5
        ));

        let mut input = InputStream::new(Cursor::new(concat!(
            "materials:\n",
            "   - name: sky\n",
            "     specular:\n",
            "         uniform: [1.2, 0.9, 3.7]\n",
            "     uniform: random # This is an inline comment\n",
            "   - name: ground\n",
            "     diffuse:\n",
            "        checkered: [BLACK, WHITE, 7.]\n",
            "     uniform: [2.1, 9.0, 7.3]\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Materials).is_ok());
        assert!(matches!(
            input.parse_materials(&var),
            Err(SceneErr::NotMatch { loc, .. }) if loc.line_num==4 && loc.col_num==9
        ))
    }

    #[test]
    fn test_transformations_parser() {
        let mut input = InputStream::new(Cursor::new(concat!(
            "transformations:\n",
            " - name: camera\n",
            "   compose:\n",
            "    - rotationz: +1\n",
            "    - translation: [-.3, 1E-02, -1E+1]\n",
        )));
        let var: Var = Var::default();
        let camera =
            rotation_z(f32::to_radians(1.0)) * translation(Vector::from((-0.3, 1e-2, -1e1)));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Transformations).is_ok());
        assert!(
            matches!(input.parse_transformations(&var), Ok(trs) if matches!(trs.get("camera"), Some(cam) if *cam==camera)
            )
        );

        let mut input = InputStream::new(Cursor::new(concat!(
            "transformations:\n",
            "  - name: rot_x\n",
            "    compose:\n",
            "      - rotationx: 90\n",
            "  - name: rot_y\n",
            "    compose:\n",
            "      - rotationy: 180\n",
            "  - name: rot_z\n",
            "    compose:\n",
            "      - rotationz: 270\n",
        )));
        let rot_x = rotation_x(f32::to_radians(90.));
        let rot_y = rotation_y(f32::to_radians(180.));
        let rot_z = rotation_z(f32::to_radians(270.));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Transformations).is_ok());
        let transformations = input.parse_transformations(&var);
        assert!(transformations.is_ok());
        assert!(matches!(transformations.as_ref().unwrap().get("rot_x"), Some(rx) if *rx==rot_x));
        assert!(matches!(transformations.as_ref().unwrap().get("rot_y"), Some(ry) if *ry==rot_y));
        assert!(matches!(transformations.as_ref().unwrap().get("rot_z"), Some(rz) if *rz==rot_z));

        let mut input = InputStream::new(Cursor::new(concat!(
            "transformations:\n",
            "  - name: rotationx\n",
            "    compose:\n",
            "      - rotationx: 90\n",
            "  - name: rotationy\n",
            "    compose:\n",
            "      - rotationy: 180\n",
            "  - name: rotationz\n",
            "    compose:\n",
            "      - rotationz: 270\n",
            "  - name: rotation_tot\n",
            "    compose:\n",
            "      - rotationx\n",
            "      - rotationy\n",
            "      - rotationz\n",
            "  - name: rotation_translation\n",
            "    compose:\n",
            "      - rotation_tot\n",
            "      - translation: E3\n",
        )));
        let rot_tot = rot_x * rot_y * rot_z;
        let rot_tra = rot_tot * translation(E3);

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Transformations).is_ok());
        let transformations = input.parse_transformations(&var);
        assert!(transformations.is_ok());
        assert!(
            matches!(transformations.as_ref().unwrap().get("rotation_tot"), Some(rt) if *rt==rot_tot)
        );
        assert!(
            matches!(transformations.as_ref().unwrap().get("rotation_translation"), Some(rt) if *rt==rot_tra)
        );

        let mut input = InputStream::new(Cursor::new(concat!(
            "transformations:\n",
            "  - name: rot_scl\n",
            "    compose:\n",
            "      - rotationx: 90\n",
            "      - scaling: [2.1, 1.7, 0.5]\n",
            "  - name: rot_y\n",
            "    compose:\n",
            "      - rotationy: 180\n",
        )));
        let rot_scl = rotation_x(f32::to_radians(90.)) * scaling(Vector::from((2.1, 1.7, 0.5)));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Transformations).is_ok());
        assert!(
            matches!(input.parse_transformations(&var), Ok(trs) if matches!(trs.get("rot_scl"), Some(rs) if *rs==rot_scl)
            )
        );

        let mut input = InputStream::new(Cursor::new(concat!(
            "transformations:\n",
            "  - name: invalid\n",
            "    compose:\n",
            "      - rotationx: 90\n",
            "      - mirroring: [2.1, 1.7, 0.5]\n",
            "  - name: rot_y\n",
            "    compose:\n",
            "      - rotationy: 180\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Transformations).is_ok());
        assert!(matches!(
            input.parse_transformations(&var),
            Err(SceneErr::UndefinedIdentifier { loc, .. }) if loc.line_num==5 && loc.col_num==9
        ));

        let mut input = InputStream::new(Cursor::new(concat!(
            "transformations:\n",
            " - name: camera\n",
            "   compose:\n",
            "     - rotationz: +1\n",
            "      - translation: [-.3, 1E-02, -1E+1]\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Transformations).is_ok());
        assert!(matches!(
            input.parse_transformations(&var),
            Err(SceneErr::NotMatch { loc, .. }) if loc.line_num==4 && loc.col_num==5
        ))
    }

    #[test]
    fn test_shapes_parser() {
        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "shapes:\n",
            "     - sphere:\n",
            "       material: sphere\n",
            "       transformation: IDENTITY\n",
            "     - plane:\n",
            "       material: sky\n",
            "       transformation: rotationx\n",
        )));
        let mut var: Var = Var::default();
        let mut world = World::default();
        let rot_x = rotation_x(f32::to_radians(90.));
        let sphere = Material {
            brdf: BRDF::Diffuse(DiffuseBRDF {
                pigment: Pigment::Uniform(UniformPigment {
                    color: Color::from((0.3, 0.4, 0.8)),
                }),
            }),
            emitted_radiance: Pigment::Uniform(UniformPigment::default()),
        };
        let sky = Material {
            brdf: BRDF::Diffuse(DiffuseBRDF {
                pigment: Pigment::Uniform(UniformPigment::default()),
            }),
            emitted_radiance: Pigment::Uniform(UniformPigment {
                color: Color::from((1.0, 0.9, 0.5)),
            }),
        };
        world.add(Box::new(Sphere::new(
            Transformation::default(),
            sphere.clone(),
        )));
        world.add(Box::new(Plane::new(rot_x, sky.clone())));
        var.transformations.insert(String::from("rotationx"), rot_x);
        var.materials.insert(String::from("sphere"), sphere);
        var.materials.insert(String::from("sky"), sky);

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Shapes).is_ok());
        let shapes = input.parse_shapes(&var);
        assert!(shapes.is_ok());
        assert_eq!(format!("{:?}", shapes.unwrap()), format!("{:?}", world));

        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "shapes:\n",
            "  - sphere:\n",
            "    material: invalid\n",
            "    transformation: IDENTITY\n",
            "  - plane:\n",
            "    material: sky\n",
            "    transformation: rotationx\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Shapes).is_ok());
        assert!(matches!(
            input.parse_shapes(&var),
            Err(SceneErr::UndefinedIdentifier { loc, .. }) if loc.line_num==4 && loc.col_num==15
        ));

        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "shapes:\n",
            "  - sphere:\n",
            "    material: sphere\n",
            "    transformation: IDENTITY\n",
            "   - plane:\n",
            "     material: sky\n",
            "     transformation: rotationx\n",
        )));

        assert!(input.match_whitespaces_and_comments().is_ok());
        assert!(input.match_keyword(Keywords::Shapes).is_ok());
        assert!(matches!(
            input.parse_shapes(&var),
            Err(SceneErr::NotMatch { loc, .. }) if loc.line_num==6 && loc.col_num==3
        ));
    }

    #[test]
    fn test_scene_parser() {
        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "\n",
            "colors:\n",
            " - name: red\n",
            "   color: [1.0, 0., 0]\n",
            " - name: green\n",
            "   color: [0.0, 1., 0]\n",
            " - name: blue\n",
            "   color: [0.0, 0., 1]                 # This is an inline comment\n",
            "# This is a comment\n",
            "\n",
            "materials:\n",
            "  - name: sky\n",
            "    specular:\n",
            "      uniform: [1.2, 0.9, 3.7]\n",
            "    uniform: blue                      # This is an inline comment\n",
            "  - name: sphere\n",
            "    diffuse:\n",
            "      checkered: [BLACK, WHITE, 7.]\n",
            "    uniform: green\n",
            "  - name: from_image\n",
            "    diffuse:\n",
            "      image: '/tmp/pfm_reference'\n",
            "    uniform: red\n",
            "\n",
            "\n",
            "transformations:\n",
            "   - name: rotationx\n",
            "     compose:\n",
            "        - rotationx: 90\n",
            "   - name: rot_y\n",
            "     compose:\n",
            "        - rotationy: 180\n",
            "   - name: camera\n",
            "     compose:\n",
            "        - rotationz: 270\n",
            "\n",
            "camera:\n",
            "  type: \"perspective\"                # This is an inline comment\n",
            "  ratio: RATIO\n",
            "  distance: 2.0\n",
            "  transformation: camera\n",
            "\n",
            "shapes:\n",
            "  - sphere:\n",
            "    material: sphere\n",
            "    transformation: IDENTITY\n",
            "  - plane:\n",
            "    material: sky\n",
            "    transformation: rotationx\n",
            "  - sphere:\n",
            "    material: from_image\n",
            "    transformation: rot_y\n",
        )));
        // Build a cli
        let cli = Cli {
            aspect_ratio: 640. / 480.,
            angle_deg: 0.0,
        };
        // Build a reference hdrimage to use with image pigment
        let pfm_reference_bytes = vec![
            0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a, 0x00, 0x00,
            0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43, 0x00, 0x00, 0xc8, 0x43,
            0x00, 0x00, 0xfa, 0x43, 0x00, 0x00, 0x16, 0x44, 0x00, 0x00, 0x2f, 0x44, 0x00, 0x00,
            0x48, 0x44, 0x00, 0x00, 0x61, 0x44, 0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41,
            0x00, 0x00, 0xf0, 0x41, 0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00,
            0x70, 0x42, 0x00, 0x00, 0x8c, 0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42,
        ];
        assert!(
            BufWriter::new(File::create(Path::new("/tmp/pfm_reference")).unwrap())
                .write_all(&pfm_reference_bytes)
                .is_ok()
        );
        let pfm_reference = HdrImage::read_pfm_file(Path::new("/tmp/pfm_reference"));
        assert!(pfm_reference.is_ok());
        // Build reference scene
        let mut scene_ref = Scene::default();
        let mut world = World::default();
        let camera = Camera::Perspective(PerspectiveCamera::new(
            2.0,
            cli.aspect_ratio,
            rotation_z(f32::to_radians(270.)),
        ));
        let sphere = Material {
            brdf: BRDF::Diffuse(DiffuseBRDF {
                pigment: Pigment::Checkered(CheckeredPigment {
                    color1: BLACK,
                    color2: WHITE,
                    steps: 7,
                }),
            }),
            emitted_radiance: Pigment::Uniform(UniformPigment {
                color: Color::from((0., 1., 0.)),
            }),
        };
        let sky = Material {
            brdf: BRDF::Specular(SpecularBRDF {
                pigment: Pigment::Uniform(UniformPigment {
                    color: Color::from((1.2, 0.9, 3.7)),
                }),
                threshold_angle_rad: PI / 1800.0,
            }),
            emitted_radiance: Pigment::Uniform(UniformPigment {
                color: Color::from((0., 0., 1.)),
            }),
        };
        let from_image = Material {
            brdf: BRDF::Diffuse(DiffuseBRDF {
                pigment: Pigment::Image(ImagePigment::new(pfm_reference.unwrap())),
            }),
            emitted_radiance: Pigment::Uniform(UniformPigment {
                color: Color::from((1., 0., 0.)),
            }),
        };
        world.add(Box::new(Sphere::new(Transformation::default(), sphere)));
        world.add(Box::new(Plane::new(rotation_x(f32::to_radians(90.)), sky)));
        world.add(Box::new(Sphere::new(
            rotation_y(f32::to_radians(180.)),
            from_image,
        )));
        scene_ref.camera = Some(camera);
        scene_ref.shapes = Some(world);

        let scene = input.parse_scene(cli);
        assert!(scene.is_ok());
        assert_eq!(format!("{:?}", scene.unwrap()), format!("{:?}", scene_ref));

        let mut input = InputStream::new(Cursor::new(concat!(
            "# This is a comment\n",
            "\n",
            "colors:\n",
            " - name: red\n",
            "   color: [1.0, 0., 0]\n",
            " - name: green\n",
            "   color: [0.0, 1., 0]\n",
            " - name: blue\n",
            "   color: [0.0, 0., 1] # This is an inline comment\n",
            "# This is a comment\n",
            "\n",
            "materials:\n",
            "  - name: sphere\n",
            "    diffuse:\n",
            "      checkered: [BLACK, WHITE, 7.]\n",
            "    uniform: blue # This is an inline comment\n",
            "\n",
            "\n",
            "transformations:\n",
            "   - name: rotationx\n",
            "     compose:\n",
            "        - rotationx: 90\n",
            "   - name: rot_y\n",
            "     compose:\n",
            "        - rotationy: 180\n",
            "   - name: camera\n",
            "     compose:\n",
            "        - rotationz: 270\n",
            "\n",
            "\n",
            "shapes:\n",
            "  - sphere:\n",
            "    material: sphere\n",
            "    transformation: IDENTITY\n",
        )));

        assert!(
            matches!(input.parse_scene(cli), Err(SceneErr::NotMatch{ loc, .. }) if loc.line_num==35&& loc.col_num==1)
        );
    }
}
