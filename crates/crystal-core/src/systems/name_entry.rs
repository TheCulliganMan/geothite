//! Native naming-screen display geometry shared by production frontends.

pub const NAME_ENTRY_COLUMNS: usize = 9;
pub const NAME_ENTRY_ROWS: usize = 5;

/// Return the source naming-screen labels, including its bottom-row controls.
pub fn name_entry_layout(uppercase: bool) -> &'static [&'static str; NAME_ENTRY_ROWS] {
    if uppercase {
        &[
            "A B C D E F G H I",
            "J K L M N O P Q R",
            "S T U V W X Y Z  ",
            "- ? ! / . ,      ",
            "lower  DEL   END ",
        ]
    } else {
        &[
            "a b c d e f g h i",
            "j k l m n o p q r",
            "s t u v w x y z  ",
            "× ( ) : ; [ ] <PK> <MN>",
            "UPPER  DEL   END ",
        ]
    }
}
