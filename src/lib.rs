use nom::*;
use std::str;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_ident_test() {
        let def1 = b"EALL_DUSasb18 ";
        let (_, cid1) = c_ident(def1).unwrap();
        assert_eq!("EALL_DUSasb18", cid1);

        let def2 = b"_EALL_DUSasb18 ";
        let (_, cid2) = c_ident(def2).unwrap();
        assert_eq!("_EALL_DUSasb18", cid2);

        // identifiers must not start with digits
        let def3 = b"3EALL_DUSasb18 ";
        let cid3_result = c_ident(def3);
        assert!(cid3_result.is_err());
    }

    #[test]
    fn c_ident_vec_test() {
        let cid = b"FZHL_DUSasb18 ";
        let (_, cid1) = c_ident_vec(cid).unwrap();

        assert_eq!(vec!("FZHL_DUSasb18".to_string()), cid1);

        let cid_vec = b"FZHL_DUSasb19,xkask_3298 ";
        let (_, cid2) = c_ident_vec(cid_vec).unwrap();

        assert_eq!(
            vec!("FZHL_DUSasb19".to_string(), "xkask_3298".to_string()),
            cid2
        );
    }

    #[test]
    fn signal_test() {
        let signal_line = b" SG_ NAME : 3|2@1- (1,0) [0|0] \"x\" UFA\r\n\r\n";
        let signal1 = signal(signal_line).unwrap();
    }
    #[test]
    fn endianess_test() {
        let (_, big_endian) = endianess(b"0").expect("Failed to parse big endian");
        assert_eq!(Endianess::BigEndian, big_endian);

        let (_, little_endian) = endianess(b"1").expect("Failed to parse little endian");
        assert_eq!(Endianess::LittleEndian, little_endian);
    }

    #[test]
    fn signal_type_test() {
        let (_, multiplexer) = signal_type(b"m34920 ").expect("Failed to parse multiplexer");
        assert_eq!(SignalType::MultiplexedSignal(34920), multiplexer);

        let (_, multiplexor) = signal_type(b"M ").expect("Failed to parse multiplexor");
        assert_eq!(SignalType::Multiplexor, multiplexor);

        let (_, plain) = signal_type(b" ").expect("Failed to parse plain");
        assert_eq!(SignalType::Plain, plain);
    }

    #[test]
    fn value_type_test() {
        let (_, vt) = value_type(b"- ").expect("Failed to parse value type");
        assert_eq!(ValueType::Signed, vt);

        let (_, vt) = value_type(b"+ ").expect("Failed to parse value type");
        assert_eq!(ValueType::Unsigned, vt);
    }

    #[test]
    fn message_definition_test() {
        let def = b"BO_ 1 MCA_A1: 6 MFA
        SG_ ABC_1 : 9|2@1+ (1,0) [0|0] \"x\" XYZ_OUS
        SG_ BasL2 : 3|2@0- (1,0) [0|0] \"x\" DFA_FUS\r\n\r\n";
        let (_, message_def) = message_definition(def).expect("Failed to parse message definition");
    }

    #[test]
    fn signal_comment_test() {
        let def1 = b"CM_ SG_ 193 KLU_R_X \"This is a signal comment test\";";
        let id1 = SignalCommentId(193);
        let comment1 = DbcElement::SignalComment(
            id1,
            "KLU_R_X".to_string(),
            "This is a signal comment test".to_string(),
            false,
        );
        let (_, comment1_def) = comment(def1).expect("Failed to parse signal comment definition");
        assert_eq!(comment1, comment1_def);
    }

    #[test]
    fn message_definition_comment_test() {
        let def1 = b"CM_ BO_ 34544 XYZ \"Some Message comment\";";
        let id1 = MessageId(34544);
        let comment1 = DbcElement::MessageDefinitionComment(
            id1,
            "XYZ".to_string(),
            "Some Message comment".to_string(),
            false,
        );
        let (_, comment1_def) =
            comment(def1).expect("Failed to parse message definition comment definition");
        assert_eq!(comment1, comment1_def);
    }

    #[test]
    fn value_description_for_signal_test() {
        let def1 = b"VAL_ 837 UF_HZ_OI 255 \"NOP\" ;";
        let id = MessageId(837);
        let name = "UF_HZ_OI".to_string();
        let descriptions = vec![ValueDescription {
            a: 255.0,
            b: "NOP".to_string(),
        }];
        let value_description_for_signal1 =
            DbcElement::ValueDescriptionsForSignal(id, name, descriptions);
        let (_, value_signal_def) =
            value_descriptions(def1).expect("Failed to parse value desc for signal");
        assert_eq!(value_description_for_signal1, value_signal_def);
    }

    #[test]
    fn value_description_for_env_var_test() {
        let def1 = b"VAL_ MY_ENV_VAR 255 \"NOP\" ;";
        let name = "MY_ENV_VAR".to_string();
        let descriptions = vec![ValueDescription {
            a: 255.0,
            b: "NOP".to_string(),
        }];
        let value_env_var1 = DbcElement::ValueDescriptionsForEnvVar(name, descriptions);
        let (_, value_env_var) =
            value_descriptions(def1).expect("Failed to parse value desc for env var");
        assert_eq!(value_env_var1, value_env_var);
    }

    #[test]
    fn environment_variable_test() {
        let def1 = b"EV_ IUV: 0 [-22|20] \"mm\" 3 7 DUMMY_NODE_VECTOR0 VECTOR_XXX;";
        let nodes1 = vec![AccessNode::AccessNodeVectorXXX];
        let env_var1 = DbcElement::EnvVariable(
            "IUV".to_string(),
            EnvType::EnvTypeFloat,
            -22,
            20,
            "mm".to_string(),
            3.0,
            7,
            AccessType::DUMMY_NODE_VECTOR0,
            nodes1,
        );
        let (_, env_var) =
            environment_variable(def1).expect("Failed to parse environment variable");
        assert_eq!(env_var1, env_var);
    }

    #[test]
    fn network_node_attribute_value_test() {
        let def = b"BA_ \"AttrName\" BU_ NodeName 12;\n";
        let node = AttributeValuedForObjectType::NetworkNodeAttributeValue("NodeName".to_string(), AttributeValue::AttributeValueU64(12));
        let attr_val_exp= DbcElement::AttributeValueForObject("AttrName".to_string(), node);
        let (_, attr_val) = attribute_value_for_object(def).unwrap();
        assert_eq!(attr_val_exp, attr_val);
    }

    #[test]
    fn message_definition_attribute_value_test() {
        let def = b"BA_ \"AttrName\" BO_ 298 13;\n";
        let msg_def = AttributeValuedForObjectType::MessageDefinitionAttributeValue(MessageId(298), AttributeValue::AttributeValueU64(13));
        let attr_val_exp= DbcElement::AttributeValueForObject("AttrName".to_string(), msg_def);
        let (_, attr_val) = attribute_value_for_object(def).unwrap();
        assert_eq!(attr_val_exp, attr_val);
    }

    #[test]
    fn signal_attribute_value_test() {
        let def = b"BA_ \"AttrName\" SG_ 198 SGName 13;\n";
        let msg_def = AttributeValuedForObjectType::SignalAttributeValue(MessageId(198), "SGName".to_string(), AttributeValue::AttributeValueU64(13));
        let attr_val_exp= DbcElement::AttributeValueForObject("AttrName".to_string(), msg_def);
        let (_, attr_val) = attribute_value_for_object(def).unwrap();
        assert_eq!(attr_val_exp, attr_val);
    }

    #[test]
    fn env_var_attribute_value_test() {
        let def = b"BA_ \"AttrName\" EV_ EvName \"CharStr\";\n";
        let msg_def = AttributeValuedForObjectType::EnvVariableAttributeValue("EvName".to_string(), AttributeValue::AttributeValueCharString("CharStr".to_string()));
        let attr_val_exp= DbcElement::AttributeValueForObject("AttrName".to_string(), msg_def);
        let (_, attr_val) = attribute_value_for_object(def).unwrap();
        assert_eq!(attr_val_exp, attr_val);
    }

    #[test]
    fn raw_attribute_value_test() {
        let def = b"BA_ \"AttrName\" \"RAW\";\n";
        let msg_def = AttributeValuedForObjectType::RawAttributeValue(AttributeValue::AttributeValueCharString("RAW".to_string()));
        let attr_val_exp= DbcElement::AttributeValueForObject("AttrName".to_string(), msg_def);
        let (_, attr_val) = attribute_value_for_object(def).unwrap();
        assert_eq!(attr_val_exp, attr_val);
    }

    #[test]
    fn new_symbols_test() {
        let def =
            b"NS_ :
                NS_DESC_
                CM_
                BA_DEF_

            ";
        let symbols_exp = vec!(Symbol("NS_DESC_".to_string()), Symbol("CM_".to_string()), Symbol("BA_DEF_".to_string()));
        let (_, symbols) = new_symbols(def).unwrap();
        assert_eq!(symbols_exp, symbols);
    }

    #[test]
    fn network_node_test() {
        let def = b"BU_: ZU XYZ ABC OIU\n";
        let nodes = vec!("ZU".to_string(), "XYZ".to_string(), "ABC".to_string(), "OIU".to_string());
        let (_, node) = network_node(def).unwrap();
        let node_exp = DbcElement::NetworkNode(nodes);
        assert_eq!(node_exp, node);
    }

     #[test]
    fn envvar_data_test() {
        let def = b"ENVVAR_DATA_ SomeEnvVarData: 399;";
        let (_, envvar_data) = envvar_data(def).unwrap();
        let envvar_data_exp = DbcElement::EnvVarData("SomeEnvVarData".to_string(), 399);
        assert_eq!(envvar_data_exp, envvar_data);
    }

    #[test]
    fn attribute_default_test() {
        let def = b"BA_DEF_DEF_ \"ZUV\" \"OAL\";";
        let (_, attr_default) = attribute_default(def).unwrap();
        let attr_default_exp = DbcElement::AttributeDefault("ZUV".to_string(), AttributeValue::AttributeValueCharString("OAL".to_string()));
        assert_eq!(attr_default_exp, attr_default);
    }

    #[test]
    fn custom_attr_def_test() {
        let def_bo = b"BA_DEF_ BO_  \"BaDef1BO\" INT 0 1000000;";
        let (_, bo_def) = custom_attr_def(def_bo).unwrap();
        let bo_def_exp = DbcElement::BODef(" \"BaDef1BO\" INT 0 1000000".to_string());
        assert_eq!(bo_def_exp, bo_def);

        let def_bu = b"BA_DEF_ BU_  \"BuDef1BO\" INT 0 1000000;";
        let (_, bu_def) = custom_attr_def(def_bu).unwrap();
        let bu_def_exp = DbcElement::BUDef(" \"BuDef1BO\" INT 0 1000000".to_string());
        assert_eq!(bu_def_exp, bu_def);
    }

    #[test]
    fn version_test() {
        let def = b"VERSION \"HNPBNNNYNNNNNNNNNNNNNNNNNNNNNNNNYNYYYYYYYY>4>%%%/4>'%**4YYY///\"";
        let version_exp = Version("HNPBNNNYNNNNNNNNNNNNNNNNNNNNNNNNYNYYYYYYYY>4>%%%/4>'%**4YYY///".to_string());
        let (_, version) = version(def).unwrap();
        assert_eq!(version_exp, version);
    }

    #[test]
    fn dbc_definition_test() {
        let sample_dbc = 
        b"
        VERSION \"0.1\"

        NS_ :
            NS_DESC_
            CM_
            BA_DEF_
            BA_
            VAL_
            CAT_DEF_
            CAT_
            FILTER
            BA_DEF_DEF_
            EV_DATA_
            ENVVAR_DATA_
            SGTYPE_
            SGTYPE_VAL_
            BA_DEF_SGTYPE_
            BA_SGTYPE_
            SIG_TYPE_REF_
            VAL_TABLE_
            SIG_GROUP_
            SIG_VALTYPE_
            SIGTYPE_VALTYPE_
            BO_TX_BU_
            BA_DEF_REL_
            BA_REL_
            BA_DEF_DEF_REL_
            BU_SG_REL_
            BU_EV_REL_
            BU_BO_REL_
            SG_MUL_VAL_

        BU_: PC

        BO_ 2000 WebData_2000: 4 Vector__XXX
            SG_ Signal_8 : 24|8@1+ (1,0) [0|255] \"\" Vector__XXX
            SG_ Signal_7 : 16|8@1+ (1,0) [0|255] \"\" Vector__XXX
            SG_ Signal_6 : 8|8@1+ (1,0) [0|255] \"\" Vector__XXX
            SG_ Signal_5 : 0|8@1+ (1,0) [0|255] \"\" Vector__XXX

        BO_ 1840 WebData_1840: 4 PC
            SG_ Signal_4 : 24|8@1+ (1,0) [0|255] \"\" Vector__XXX
            SG_ Signal_3 : 16|8@1+ (1,0) [0|255] \"\" Vector__XXX
            SG_ Signal_2 : 8|8@1+ (1,0) [0|255] \"\" Vector__XXX
            SG_ Signal_1 : 0|8@1+ (1,0) [0|0] \"\" Vector__XXX

        EV_ Environment1: 0 [0|220] \"\" 0 6 DUMMY_NODE_VECTOR0 DUMMY_NODE_VECTOR2;

        EV_ Environment2: 0 [0|177] \"\" 0 7 DUMMY_NODE_VECTOR1 DUMMY_NODE_VECTOR2;

        BA_DEF_DEF_ \"BusType\" \"AS\";

        ENVVAR_DATA_ SomeEnvVarData: 399;

        CM_ SG_ 4 TestSigLittleUnsigned1 \"asaklfjlsdfjlsdfgls
        HH?=(%)/&KKDKFSDKFKDFKSDFKSDFNKCnvsdcvsvxkcv\";
        CM_ SG_ 5 TestSigLittleUnsigned1 \"asaklfjlsdfjlsdfgls
        =0943503450KFSDKFKDFKSDFKSDFNKCnvsdcvsvxkcv\";
        BA_ \"Attr\" BO_ 4358435 283;
        BA_ \"Attr\" BO_ 56949545 344;
        VAL_ 3454 TestValue 3423232 \"positive\" 359595 \"doe\" -1393 \"john\" ;
        VAL_ 3454 TestValue 3423232 \"positive\" 359595 \"doe\" -1393 \"positive\" 359595 \"doe\" -1393 \"john\" ;
        ";

        let (remaining, dbc_def) = dbc_definition(sample_dbc).unwrap();

        println!("Remaining {:?}\nResult {:?}", str::from_utf8(remaining).unwrap(), dbc_def);
    }
}

#[derive(Debug, PartialEq)]
pub struct Dbc(String);

#[derive(Debug, PartialEq)]
pub struct Label(String);

#[derive(Debug, PartialEq)]
pub struct Signal {
    name: String,
    signal_type: SignalType,
    offset: u64,
    length: u64,
    endianess: Endianess,
    value_type: ValueType,
    slope: f64,
    intercept: f64,
    min: f64,
    max: f64,
    unit: String,
    receivers: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct SignalCommentId(u64);

#[derive(Debug, PartialEq)]
pub struct MessageId(u64);

#[derive(Debug, PartialEq)]
pub struct MessageTransmitter(String);

#[derive(Debug, PartialEq)]
pub struct Version(String);

#[derive(Debug, PartialEq)]
pub struct Symbol(String);

#[derive(Debug, PartialEq)]
pub enum SignalType {
    /// Multiplexor switch
    Multiplexor,
    /// Signal us being multiplexed by the multiplexer switch.
    MultiplexedSignal(u64),
    /// Normal signal
    Plain,
}

#[derive(Debug, PartialEq)]
pub enum Endianess {
    LittleEndian,
    BigEndian,
}

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Signed,
    Unsigned,
}

#[derive(Debug, PartialEq)]
pub enum EnvType {
    EnvTypeFloat,
    EnvTypeu64,
    EnvTypeData,
}

#[derive(Debug, PartialEq)]
pub struct ECUDef(String);

type KeyValue = (String, i64);

#[derive(Debug, PartialEq)]
pub struct LabelDescription {
    id: u64,
    signal_name: String,
    labels: Vec<Label>,
    extended: bool,
}

#[derive(Debug, PartialEq)]
pub enum Transmitter {
    TransmitterNodeName(String),
    TransmitterNoSender,
}

#[derive(Debug, PartialEq)]
pub enum AccessType {
    DUMMY_NODE_VECTOR0,
    DUMMY_NODE_VECTOR1,
    DUMMY_NODE_VECTOR2,
    DUMMY_NODE_VECTOR3,
}

#[derive(Debug, PartialEq)]
pub enum AccessNode {
    AccessNodeVectorXXX,
    AccessNodeName(String),
}

#[derive(Debug, PartialEq)]
pub enum SignalAttributeValue {
    Text(String),
    Int(i64),
}

#[derive(Debug, PartialEq)]
pub enum AttributeValuedForObjectType {
    RawAttributeValue(AttributeValue),
    NetworkNodeAttributeValue(String, AttributeValue),
    MessageDefinitionAttributeValue(MessageId, AttributeValue),
    SignalAttributeValue(MessageId, String, AttributeValue),
    EnvVariableAttributeValue(String, AttributeValue),
}

#[derive(Debug, PartialEq)]
pub enum AttributeValueType {
    AttributeValueTypeInt(i64, i64),
    AttributeValueTypeHex(i64, i64),
    AttributeValueTypeFloat(f64, f64),
    AttributeValueTypeString,
    AttributeValueTypeEnum(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub struct ValueDescription {
    a: f64,
    b: String,
}

#[derive(Debug, PartialEq)]
pub struct AttributeDefault {
    name: String,
    value: AttributeValue,
}

#[derive(Debug, PartialEq)]
pub enum AttributeValue {
    AttributeValueU64(u64),
    AttributeValueI64(i64),
    AttributeValueF64(f64),
    AttributeValueCharString(String),
}

#[derive(Debug, PartialEq)]
pub enum DbcElement {
    ValueTable(String, Vec<KeyValue>),
    NetworkNodeComment(String),
    MessageDefinitionComment(MessageId, String, String, bool),
    SignalComment(SignalCommentId, String, String, bool),
    EnvVarComment(String),
    Message(MessageId, bool, String, u64, String, Vec<Signal>),
    EnvVariable(
        String,
        EnvType,
        i64,
        i64,
        String,
        f64,
        i64,
        AccessType,
        Vec<AccessNode>,
    ),
    BitTimingSection,
    EnvVarData(String, u64),
    NetworkNode(Vec<String>),
    ///
    /// BA_DEF_DEF
    ///
    AttributeDefault(String, AttributeValue),
    AttributeValueForObject(String, AttributeValuedForObjectType),
    Def,
    ///
    /// BA_DEF BO
    ///
    BODef(String),
    ///
    /// BA_DEF BU
    ///
    BUDef(String),
    ValueDescriptionsForSignal(MessageId, String, Vec<ValueDescription>),
    ValueDescriptionsForEnvVar(String, Vec<ValueDescription>),
}

#[derive(Debug, PartialEq)]
pub struct DbcDefinition {
    version: Version,
    symbols: Vec<Symbol>,
    elements: Vec<DbcElement>,
}

fn is_colon(chr: char) -> bool {
    chr == ':'
}

fn is_semi_colon(chr: char) -> bool {
    chr == ';'
}

fn is_space_s(chr: char) -> bool {
    chr == ' '
}

fn is_c_string_char(chr: char) -> bool {
    chr.is_digit(10) || chr.is_alphabetic() || chr == '_'
}

fn is_c_ident_head(chr: char) -> bool {
   chr.is_alphabetic() || chr == '_'
}

fn is_quote(chr: char) -> bool {
    chr == '"'
}

/// Single space
named!(ss<char>, char!(' '));

/// Colon
named!(colon<char>, char!(':'));

/// Comma aka ','
named!(comma<char>, char!(','));

/// Comma aka ';'
named!(semi_colon<char>, char!(';'));

/// Quote aka '"'
named!(quote<char>, char!('"'));

named!(pipe<char>, char!('|'));

named!(at<char>, char!('@'));

/// brace open aka '('
named!(brc_open<char>, char!('('));

/// brace close aka '('
named!(brc_close<char>, char!(')'));

/// bracket open aka '['
named!(brk_open<char>, char!('['));

/// bracket close aka ']'
named!(brk_close<char>, char!(']'));

/// A valid C_identifier. C_identifiers start with a  alphacharacter or an underscore
/// and may further consist of alpha­numeric, characters and underscore
named!(c_ident<&str>,
    map_res!(
        recognize!(
            do_parse!(
                take_while1!(|x| is_c_ident_head(x as char))  >>
                take_while!(|x| is_c_string_char(x as char)) >>
                ()
            )
        ),
        str::from_utf8
    )
);

named!(c_ident_vec<Vec<&str>>, separated_nonempty_list!(comma, c_ident));

named!(u64_s<u64>, map_res!(
        digit,
        |s| std::str::FromStr::from_str(str::from_utf8(s).unwrap())
    )
 );

named!(i64_digit<i64>,
    flat_map!(recognize!(tuple!(opt!(alt!(char!('+') | char!('-'))), digit)), parse_to!(i64))
);

named!(quoted<&str>,
    do_parse!(
            quote                                 >>
        s:  take_till_s!(|c |is_quote(c as char)) >>
            quote                                 >>
        (str::from_utf8(s).unwrap())
    )
);

named!(pub little_endian<Endianess>, map!(char!('1'), |_| Endianess::LittleEndian));

named!(pub big_endian<Endianess>, map!(char!('0'), |_| Endianess::BigEndian));

named!(pub endianess<Endianess>, alt!(little_endian | big_endian));

named!(pub message_id<MessageId>, map!(u64_s, MessageId));

named!(pub signal_comment_id<SignalCommentId>, map!(u64_s, SignalCommentId));

named!(pub signed<ValueType>, map!(char!('-'), |_| ValueType::Signed));

named!(pub unsigned<ValueType>, map!(char!('+'), |_| ValueType::Unsigned));

named!(pub value_type<ValueType>, alt!(signed | unsigned));

named!(pub multiplexer<SignalType>,
    do_parse!(
           char!('m') >>
        d: u64_s      >>
           ss         >>
        (SignalType::MultiplexedSignal(d))
    )
);

named!(pub multiplexor<SignalType>,
    do_parse!(
        char!('M') >>
        ss         >>
        (SignalType::Multiplexor)
    )
);

named!(pub plain<SignalType>,
    do_parse!(
        ss >>
        (SignalType::Plain)
    )
);

named!(pub version<Version>,
    do_parse!(
           tag!("VERSION") >>
           ss              >>
        v: quoted          >>
        (Version(v.to_string()))
    )
);

named!(pub signal_type<SignalType>, alt!(multiplexer | multiplexor | plain));

named!(pub signal<Signal>,
    do_parse!(
                             space >>
                             tag!("SG_") >>
                             ss          >>
       name:                 c_ident     >>
       signal_type:          signal_type >>
                             colon       >>
                             ss          >>
       offset:               u64_s       >>
                             pipe        >>
       length:               u64_s       >>
                             at          >>
       endianess:            endianess   >>
       value_type:           value_type  >>
                             ss          >>
                             brc_open    >>
       slope:                double      >>
                             comma       >>
       intercept:            double      >>
                             brc_close   >>
                             ss          >>
                             brk_open    >>
       min:                  double      >>
                             pipe        >>
       max:                  double      >>
                             brk_close   >>
                             ss          >>
       unit:                 quoted      >>
                             ss          >>
       receivers:            c_ident_vec >>
        (Signal {
            name: name.to_string(),
            signal_type: signal_type,
            offset: offset,
            length: length,
            endianess: endianess,
            value_type: value_type,
            slope: slope,
            intercept: intercept,
            min: min,
            max: max,
            unit:  unit.to_string(),
            receivers: receivers.iter().map(|s| s.to_string()).collect(),
        })
    )
);

named!(pub message_definition<DbcElement>,
  do_parse!(
                 tag!("BO_")                                                                 >>
                 ss                                                                          >>
    id:          message_id                                                                  >>
                 ss                                                                          >>
    name:        map!(take_till_s!(|c| is_colon(c as char)), |s| str::from_utf8(s).unwrap()) >>
                 colon                                                                       >>
                 ss                                                                          >>
    size:        u64_s                                                                       >>
                 ss                                                                          >>
    transmitter: c_ident                                                                     >>
                 line_ending                                                                 >>
    signals: separated_nonempty_list!(line_ending, signal)                                   >>
    (DbcElement::Message(id, false, name.to_string(), size, transmitter.to_string(), signals))
  )
);

named!(pub attribute_default<DbcElement>,
    do_parse!(
                         tag!("BA_DEF_DEF_") >>
                         ss                  >>
        attribute_name:  quoted              >>
                         ss                  >>
        attribute_value: attribute_value     >>
                         semi_colon          >>
        (DbcElement::AttributeDefault(attribute_name.to_string(), attribute_value))
    )
);

named!(pub signal_comment<DbcElement>,
    do_parse!(
                 tag!("SG_")       >>
                 ss                >>
        id:      signal_comment_id >>
                 ss                >>
        name:    c_ident           >>
                 ss                >>
        comment: quoted            >>
        (DbcElement::SignalComment(id, name.to_string(), comment.to_string(), false))
    )
);

named!(pub message_definition_comment<DbcElement>,
    do_parse!(
                  tag!("BO_")                                                                   >>
                  ss                                                                            >>
        id:       message_id                                                                    >>
                  ss                                                                            >>
                  // TODO not only c ident ?
        name:     map!(take_till_s!(|c| is_space_s(c as char)), |s| str::from_utf8(s).unwrap()) >>
                  ss                                                                            >>
        comment: quoted                                                                         >>
        (DbcElement::MessageDefinitionComment(id, name.to_string(), comment.to_string(), false))
    )
);

named!(pub comment<DbcElement>,
    do_parse!(
           tag!("CM_")                                       >>
           ss                                                >>
        c: alt!(signal_comment | message_definition_comment) >>
           semi_colon                                        >>
        (c)
    )
);

named!(pub value_description<ValueDescription>,
    do_parse!(
        a: double >>
           ss     >>
        b: quoted >>
        (ValueDescription { a: a, b: b.to_string() })
    )
);

named!(pub value_description_for_signal<DbcElement>,
    do_parse!(
              tag!("VAL_")                                                                     >>
              ss                                                                               >>
        id:   message_id                                                                       >>
              ss                                                                               >>
        name: c_ident                                                                          >>
        descriptions:  many_till!(preceded!(ss, value_description), preceded!(ss, semi_colon)) >>
        (DbcElement::ValueDescriptionsForSignal(id, name.to_string(), descriptions.0))
    )
);

named!(pub value_description_for_env_var<DbcElement>,
    do_parse!(
                      tag!("VAL_")                                                            >>
                      ss                                                                      >>
        name:         c_ident                                                                 >>
        descriptions: many_till!(preceded!(ss, value_description), preceded!(ss, semi_colon)) >>
        (DbcElement::ValueDescriptionsForEnvVar(name.to_string(), descriptions.0))
    )
);

named!(pub value_descriptions<DbcElement>,
    alt!(value_description_for_signal | value_description_for_env_var)
);

named!(env_float<EnvType>, value!(EnvType::EnvTypeFloat, char!('0')));
named!(env_int<EnvType>, value!(EnvType::EnvTypeu64, char!('1')));
named!(env_data<EnvType>, value!(EnvType::EnvTypeData, char!('2')));

/// 9 Environment Variable Definitions
named!(pub env_var_type<EnvType>, alt!(env_float | env_int | env_data));

named!(dummy_node_vector_0<AccessType>, value!(AccessType::DUMMY_NODE_VECTOR0, char!('0')));
named!(dummy_node_vector_1<AccessType>, value!(AccessType::DUMMY_NODE_VECTOR1, char!('1')));
named!(dummy_node_vector_2<AccessType>, value!(AccessType::DUMMY_NODE_VECTOR2, char!('2')));
named!(dummy_node_vector_3<AccessType>, value!(AccessType::DUMMY_NODE_VECTOR3, char!('3')));

/// 9 Environment Variable Definitions
named!(pub access_type<AccessType>,
    do_parse!(
              tag!("DUMMY_NODE_VECTOR") >>
        node: alt!(dummy_node_vector_0 | dummy_node_vector_1 | dummy_node_vector_2 | dummy_node_vector_3) >>
        (node)
    )
);

named!(access_node_vector_xxx<AccessNode>,  value!(AccessNode::AccessNodeVectorXXX, tag!("VECTOR_XXX")));
named!(access_node_name<AccessNode>,  map!(c_ident, |name| AccessNode::AccessNodeName(name.to_string())));

/// 9 Environment Variable Definitions
named!(pub access_node<AccessNode>, alt!(access_node_vector_xxx | access_node_name));

/// 9 Environment Variable Definitions
named!(environment_variable<DbcElement>,
    do_parse!(
                       tag!("EV_")                                  >>
                       ss                                           >>
        name:          c_ident                                      >>
                       colon                                        >>
                       ss                                           >>
        type_:         env_var_type                                 >>
                       ss                                           >>
                       brk_open                                     >>
        min:           i64_digit                                    >>
                       pipe                                         >>
        max:           i64_digit                                    >>
                       brk_close                                    >>
                       ss                                           >>
        unit:          quoted                                       >>
                       ss                                           >>
        initial_value: double                                       >>
                       ss                                           >>
        id:            i64_digit                                    >>
                       ss                                           >>
        access_type:   access_type                                  >>
                       ss                                           >>
        access_nodes:  separated_nonempty_list!(comma, access_node) >>
                       semi_colon >>
       (DbcElement::EnvVariable(name.to_string(), type_, min, max, unit.to_string(), initial_value, id, access_type, access_nodes))
    )
);

named!(pub envvar_data<DbcElement>,
    do_parse!(
                      tag!("ENVVAR_DATA_") >>
                      ss                   >>
        env_var_name: c_ident              >>
                      colon                >>
                      ss                   >>
        data_size:    u64_s                >>
                      semi_colon           >>
        (DbcElement::EnvVarData(env_var_name.to_string(), data_size))
    )
);

named!(pub attribute_value_uint64<AttributeValue>, 
    map!(u64_s, AttributeValue::AttributeValueU64)
);

named!(pub attribute_value_int64<AttributeValue>, 
    map!(i64_digit, AttributeValue::AttributeValueI64)
);

named!(pub attribute_value_f64<AttributeValue>, 
    map!(double, AttributeValue::AttributeValueF64)
);

named!(pub attribute_value_charstr<AttributeValue>, 
    map!(quoted, |x| AttributeValue::AttributeValueCharString(x.to_string()))
);

named!(pub attribute_value<AttributeValue>,
    alt!(
        attribute_value_uint64 |
        attribute_value_int64  |
        attribute_value_f64    |
        attribute_value_charstr
    )
);

named!(pub network_node_attribute_value<AttributeValuedForObjectType>,
    do_parse!(
                   tag!("BU_")     >>
                   ss              >>
        node_name: c_ident         >>
                   ss              >>
        value:     attribute_value >>
        (AttributeValuedForObjectType::NetworkNodeAttributeValue(node_name.to_string(), value))
    )
);

named!(pub message_definition_attribute_value<AttributeValuedForObjectType>,
    do_parse!(
                    tag!("BO_")     >>
                    ss              >>
        message_id: message_id      >>
                    ss              >>
        value:      attribute_value >>
        (AttributeValuedForObjectType::MessageDefinitionAttributeValue(message_id, value))
    )
);

named!(pub signal_attribute_value<AttributeValuedForObjectType>,
    do_parse!(
                     tag!("SG_")     >>
                     ss              >>
        message_id:  message_id      >>
                     ss              >>
        signal_name: c_ident         >>
                     ss              >>
        value:       attribute_value >>
        (AttributeValuedForObjectType::SignalAttributeValue(message_id, signal_name.to_string(), value))
    )
);

named!(pub env_variable_attribute_value<AttributeValuedForObjectType>,
    do_parse!(
                      tag!("EV_")     >>
                      ss              >>
        env_var_name: c_ident         >>
                      ss              >>
        value:        attribute_value >>
        (AttributeValuedForObjectType::EnvVariableAttributeValue(env_var_name.to_string(), value))
    )
);

named!(pub raw_attribute_value<AttributeValuedForObjectType>,
    map!(attribute_value, AttributeValuedForObjectType::RawAttributeValue)
);

named!(pub attribute_value_for_object<DbcElement>,
    do_parse!(
               tag!("BA_") >>
               ss          >>
        name:  quoted      >>
               ss          >>
        value: alt!(
                    network_node_attribute_value       |
                    message_definition_attribute_value |
                    signal_attribute_value             |
                    env_variable_attribute_value       |
                    raw_attribute_value
                )          >>
                semi_colon >>
        (DbcElement::AttributeValueForObject(name.to_string(), value))
    )
);

named!(pub bu_def<DbcElement>,
    do_parse!(
           tag!("BU_") >>
           ss          >>
        x: map!(take_till_s!(|c |is_semi_colon(c as char)), |x| str::from_utf8(x).unwrap()) >>
        (DbcElement::BUDef(x.to_string()))
    )
);

named!(pub bo_def<DbcElement>,
    do_parse!(
           tag!("BO_") >>
           ss          >>
        x: map!(take_till_s!(|c |is_semi_colon(c as char)), |x| str::from_utf8(x).unwrap()) >>
        (DbcElement::BODef(x.to_string()))
    )
);

named!(pub custom_attr_def<DbcElement>,
    do_parse!(
        tag!("BA_DEF_")            >>
        ss                         >>
        def: alt!(bu_def | bo_def) >>
        (def)
    )
);

named!(pub symbol<Symbol>,
    do_parse!(
                space   >>
        symbol: c_ident >>
        (Symbol(symbol.to_string()))
    )
);

named!(pub new_symbols<Vec<Symbol>>,
    do_parse!(
                 tag!("NS_ :")                                 >>
                 line_ending                                   >>
        symbols: separated_nonempty_list!(line_ending, symbol) >>
        (symbols)
    )
);

named!(pub network_node<DbcElement>,
    do_parse!(
            tag!("BU_:") >>
            ss           >>
        li: map!(separated_nonempty_list!(ss, c_ident), |li| li.iter().map(|s| s.to_string()).collect())>>
        (DbcElement::NetworkNode(li))
    )
);

named!(pub dbc_element<DbcElement>,
    alt!(
        message_definition            |
        environment_variable          |
        comment                       |
        value_description_for_env_var |
        value_description_for_signal  |
        attribute_value_for_object    |
        network_node                  |
        envvar_data                   |
        attribute_default
    )
);

named!(pub dbc_definition<DbcDefinition>,
    do_parse!(
                   opt!(multispace)                                          >>
        version:   version                                                   >>
                   opt!(multispace)                                          >>
        symbols:   new_symbols                                               >>
                   opt!(multispace)                                          >>
        elements: separated_nonempty_list_complete!(multispace, dbc_element) >>
        (DbcDefinition { version, symbols, elements })
    )
);
