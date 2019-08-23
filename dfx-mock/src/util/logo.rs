use console::Style;
use regex::Regex;

pub fn generate_logo() -> String {
    let blue_re = Regex::new(r"b\{([^}]+)\}").unwrap();
    let magenta_re = Regex::new(r"m\{([^}]+)\}").unwrap();
    let red_re = Regex::new(r"r\{([^}]+)\}").unwrap();
    let orange_re = Regex::new(r"o\{([^}]+)\}").unwrap();
    let yellow_re = Regex::new(r"y\{([^}]+)\}").unwrap();

    let blue = Style::new().blue().bold();
    let magenta = Style::new().red();
    let red = Style::new().red().bold();
    let orange = Style::new().yellow();
    let yellow = Style::new().yellow().bold();

    // TODO: Add colors.
    let logo = r###"
               b{*////////*}                              o{./((((((((*}
           b{//////////////////}                      o{,((((((((((((((}y{////}
        b{////////////////////////}                o{,((((((((((((((}y{//////////}
      b{/////}m{(######(}b{///////////////.}           o{/((((((((((((((}y{/////////////*}
    b{.//}m{########}         b{////////////,}       o{((((((((((((}         y{./////////*}
   b{,/}m{########}              b{///////////}    o{/((((((((((}               y{/////****}
   m{########}                  b{*//////////*}o{((((((((((}                   y{/*******}
  m{########}                     b{*//////////}o{((((((/}                      y{*******,}
  m{#######,}                       b{*//////////}o{(((}                        y{********}
  m{#######}                          r{/}b{/////////,}                         y{.*******}
  m{#######.}                        r{///}b{//////////.}                       y{********}
  m{########}                      r{//////}b{///////////}                      y{*******,}
   m{####}r{((((}                   r{/(///////}b{/,//////////}                   y{********}
   m{/##}r{((((((/               //////////*}   b{*//////////*}              y{********}b{/}
    r{.((((((((((         .////////////}       b{*////////////}        y{.********}b{//}
      r{((((((((((((((((///////////(/}           b{*//////////////}y{*********}b{/////}
        r{((((((((((((////////////.}               b{.////////////////////////}
          r{,(((((((///////////,}                     b{,//////////////////.}
              r{.(////////(.}                             b{./////////*}
"###.to_string();

    let logo = blue_re.replace_all(&logo, format!("{}", blue.apply_to("$1").to_owned()).as_str());
    let logo = magenta_re.replace_all(&logo, format!("{}", magenta.apply_to("$1").to_owned()).as_str());
    let logo = red_re.replace_all(&logo, format!("{}", red.apply_to("$1").to_owned()).as_str());
    let logo = orange_re.replace_all(&logo, format!("{}", orange.apply_to("$1").to_owned()).as_str());
    let logo = yellow_re.replace_all(&logo, format!("{}", yellow.apply_to("$1").to_owned()).as_str());

    logo.to_string()
}
