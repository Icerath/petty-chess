use std::fmt;

use crate::prelude::*;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Zobrist(u64);

impl Zobrist {
    pub const DEFAULT: Self = Self(0);
    #[inline]
    pub fn xor_side_to_move(&mut self) {
        self.0 ^= SIDE_KEY;
    }
    #[inline]
    pub fn xor_piece(&mut self, sq: Square, piece: Piece) {
        self.0 ^= PIECE_KEYS[piece as usize][sq];
    }
    #[inline]
    pub fn xor_can_castle(&mut self, can_castle: CanCastle) {
        self.0 ^= CASTLE_KEYS[can_castle.bits() as usize];
    }
    #[inline]
    pub fn xor_en_passant(&mut self, sq: Square) {
        self.0 ^= EN_PASSANT_KEYS[sq];
    }
}

impl fmt::Debug for Zobrist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[allow(clippy::unreadable_literal)]
static SIDE_KEY: u64 = 3988773030008317031;

#[allow(clippy::unreadable_literal)]
static CASTLE_KEYS: [u64; 16] = [
    4000891911605153678,
    2104839265066132248,
    16117529730937881096,
    1421746797307350215,
    1755358467280046521,
    6338867737573270586,
    1981052929136995173,
    18429876352905917621,
    6561643224553369294,
    7781244910791909337,
    13307555969480783050,
    9451674631511907449,
    6837815413783026016,
    14725854867246430531,
    11763285877110803639,
    17136652257074978461,
];

#[allow(clippy::unreadable_literal)]
static EN_PASSANT_KEYS: [u64; 64] = [
    1042276050105134611,
    6566006726995064568,
    1514999941593874607,
    15350484710731604754,
    10524328968245728991,
    11716172076496245516,
    1379872687972825765,
    2506796259222419669,
    14629667652995659734,
    116529131024005113,
    10287305575215680174,
    13657709554245089983,
    13632664766982451663,
    8563630829173919571,
    10847121705678002654,
    9726349842372265910,
    11529687466435003550,
    6192062754586046521,
    11130103793264464988,
    9868279292926297372,
    5274954524483717976,
    9836777418375690369,
    1066250829016362356,
    2498005924144091993,
    1367713871355795621,
    11950210842508855899,
    6731563284289841860,
    8534024648230760555,
    14335618270223385107,
    3151437542929116193,
    10696409025942204057,
    7858067369594360577,
    8631502994082082556,
    17443658256441160001,
    4861231121381119516,
    8247114974569134462,
    10429381417137299980,
    8886292779001611345,
    13463454089045639058,
    2874401831344367094,
    13235232940021297182,
    11270170389750510095,
    1816349173647753193,
    8310112147642101157,
    2581676467345127425,
    279712843591953658,
    13927694278634128775,
    16524082681551964979,
    11897398371292159099,
    5999203780063126274,
    14195781244320288393,
    15366598745813156972,
    1099124345343988,
    10441337223563603518,
    7353460738670693699,
    4438553506318672686,
    714733236752466549,
    6644277755121294626,
    16762976354106365008,
    171646636667009511,
    9573138623275261968,
    1911503050713787755,
    13813663823403383853,
    9992727740021666685,
];

#[allow(clippy::unreadable_literal)]
static PIECE_KEYS: [[u64; 64]; 12] = [
    [
        3224174009003844652,
        13541544634954816126,
        5132307558759830092,
        13151597882092509248,
        12182899764966871001,
        8085587600843061940,
        15438408257864350730,
        8962770071628535284,
        14325624181985568817,
        14158466888770072874,
        6060784355498417625,
        10309914699248262598,
        17064576848981378498,
        17670047967696164823,
        6417726961271637387,
        2263003683078771483,
        13823144711064873418,
        1993369457644632265,
        14299205885590473935,
        6598971081347667511,
        10486796033237383307,
        3239543990706009023,
        14394974530623751910,
        722740555340684078,
        13452130693823559392,
        17370514318291380359,
        18416951753181328351,
        16812997971124372665,
        9020832346988319670,
        10667560700644574962,
        14871860066789147310,
        9515361529702433801,
        14212794117657183484,
        7124535138396150211,
        651631577403600062,
        11755402050908465554,
        2737395703783771302,
        18423163814886963842,
        17212206003288973711,
        12155778007943658795,
        12069022389727001824,
        11000988201012960476,
        2728641933693126409,
        15731166379248768384,
        15109981087756305962,
        10143725465978876758,
        8048066763818506364,
        16323636401541105626,
        17128706663326990501,
        12375539228545506219,
        13254824014682780271,
        5345319620831053175,
        13094942882470075467,
        9133935223441280449,
        13285138014140554436,
        8658474831702763367,
        1705931291679401544,
        10876701435712819605,
        2280505305981054483,
        15906113119216022308,
        8422340715307064345,
        9825438086163412227,
        12659510483379897798,
        15363612655945410301,
    ],
    [
        3511911597811926434,
        6327978018168999374,
        15637930665636820672,
        9401793921081450480,
        14284359749183063716,
        2129974051312038461,
        12201089377725098847,
        779649218993963137,
        14286552029848351127,
        2530606492173057067,
        6912778733804916221,
        13482643146412399041,
        227653417878062023,
        8600544096072999295,
        13883532739759827377,
        684968238157805688,
        5254316886855330975,
        6539104181984314961,
        9982325588033474498,
        15167679676437236577,
        15014802530165791977,
        3968636877006648072,
        11516583992603316496,
        3945598120575069335,
        18129305834082760058,
        5527639303781902388,
        16027473102427851402,
        514333975428424269,
        14282248841683744232,
        5298433322167386187,
        8503549048592897241,
        9538004679021907152,
        10852149007808381520,
        1678801338176604147,
        10996120209304638118,
        15607883925631126101,
        4863882105269002152,
        17085667093051440512,
        8150470709861036618,
        6553910239019572954,
        1263303174677446204,
        247184867321108517,
        1690203231022444468,
        16176738222163001432,
        7527185101574196740,
        15970510003655702707,
        1491599281939614612,
        17533945268976846648,
        5273860414045746434,
        446395916665117552,
        17654994372544914476,
        13655278784366485161,
        25702926615914419,
        16052400785010370068,
        2437454849202883891,
        2703976148247451403,
        2518494400702891399,
        825179857950696656,
        16468640254341227415,
        6277987850455462243,
        7366889854811827442,
        6589627663501455765,
        6658207660968236658,
        12281708271999221976,
    ],
    [
        1557072045878044134,
        12234729018688808951,
        2884133720392167559,
        15535314612866187046,
        15566500177554246660,
        4142378017367299104,
        14023227719432223576,
        16076462252814559,
        13522447858787764178,
        13767766423804016648,
        7589878597087182822,
        16922517606329971402,
        11934437802580476483,
        381540742156348534,
        17555902984893178457,
        11556492694354110145,
        1780703360467180303,
        1542516716549993701,
        17520990630602070349,
        5551376872009272265,
        1892234305995390885,
        8826998587972569849,
        6196904723042572849,
        17074870153780841300,
        11492122516976018209,
        953861723728689033,
        11238684578536863706,
        4425693022265462869,
        10470829810546027365,
        3396589446983630370,
        17660628780154523390,
        6241764282864750782,
        11422632674249536768,
        9416823650723545516,
        9832784536073845284,
        18198750315559574774,
        14476424591912977725,
        14670302553021683571,
        2666680214396834975,
        15524973357574972204,
        624090707025181248,
        4890154064066237651,
        4991088373137093108,
        9076198188999943677,
        1631176414010355008,
        8394985298071783121,
        13982365702158499532,
        13944710541649777205,
        5573717257586349015,
        16834669805650051404,
        486772234484300447,
        1223580111367452757,
        11136980246101433644,
        7413294148505995506,
        6569437803542849516,
        16158903957194366899,
        9152920368787471105,
        5314528874336935009,
        8124026154384039472,
        6173567145367219787,
        14163093880903068421,
        10886953147612578868,
        8468278435426703704,
        16531404095974155718,
    ],
    [
        6560073386180015236,
        4114060352844849754,
        9454485285842973428,
        15223555316717958037,
        11718904856906475555,
        6260930083317306660,
        13473347709592570985,
        8789155340607720988,
        5724481857299159393,
        3759641986462618437,
        1876800031026681477,
        10395995698986520678,
        13764822462289634871,
        9855983229201501950,
        7788801916780466305,
        9143097545298983502,
        17588995890447031916,
        9384072183553497475,
        6685962699615447646,
        5207743687632006033,
        7399880492533936794,
        15290506957668835093,
        11650703789483423641,
        9549191687444639866,
        9467242725702426386,
        7739686271786671259,
        7494276340292014810,
        4444745051383469103,
        13510793968802176028,
        276641608177284339,
        16961848238724482847,
        16277362846234531071,
        15983098308903082430,
        9543691382357075974,
        5955924588262142412,
        12107183959270983344,
        7609359792378052748,
        13579747929306554311,
        2934226780533947827,
        14090588126939650479,
        17614876283676005012,
        3048817337503868481,
        10586415877630181737,
        6970253707774851414,
        17282633952041881848,
        4287423338283982188,
        3732627296118271306,
        15539751498026388136,
        9460939228834518955,
        13571760892890665275,
        9326983915205034696,
        3841294666824343383,
        7986128012847066990,
        16809160999154074783,
        9644510549971380154,
        18050581111081288534,
        11348224295102168841,
        8604968014324855519,
        7032473814462001564,
        10587202618742515504,
        5964959917299503092,
        9267284930573072719,
        4152265896536148647,
        2136184260116379991,
    ],
    [
        1229414586039840664,
        6962534197554896683,
        6116737886960762797,
        8797656079439717481,
        3509001141542083797,
        10060379134633406022,
        5270611749666292802,
        15162578031992454325,
        1458547862036568831,
        16439881441919321685,
        2276455351882832375,
        9969802551350587270,
        14977444127196184081,
        6285018852501204797,
        3697953825408308378,
        4677878345374084936,
        6749733911947091366,
        4622692079564618059,
        6009593980967030517,
        16552509508186957114,
        10781147333757319300,
        5445571562271957224,
        4964681884649312075,
        16644774721737285568,
        8968904891493447898,
        13080605631626710609,
        7388028481166588742,
        13347267908670108917,
        13960214188486285375,
        8701096975787799099,
        5360815788566901717,
        680847935261537901,
        8253224599274762345,
        8507248243441267482,
        15363458166752743135,
        2675269577110028873,
        16212526923527994983,
        7112073526485346751,
        13260798920128411396,
        369566956421252988,
        3541695611295165512,
        8134467101550781248,
        14001625427675427495,
        4415664006934789386,
        8458759472514221789,
        10179869106181385692,
        4998592326042449256,
        7200217949100554223,
        8305856928483221945,
        18106947117107821094,
        16942546696791499999,
        12872008113102179253,
        10311707949958510256,
        3949055606765284042,
        5217320376794011013,
        4009189707937114817,
        2115141242393419936,
        7038214549700405634,
        15119375038610217472,
        12331632433396951962,
        6991853355414028456,
        14706430468502368792,
        11850251772765677652,
        5043264869840845549,
    ],
    [
        12957263245306540569,
        14271122841966011690,
        5068627615713943479,
        18433550260411840724,
        2359697121437305837,
        8496956195943641412,
        17505041312818692816,
        14203071452281129021,
        18127614187944720445,
        11661966607847437070,
        7715267776159803981,
        6564401727770735092,
        14078131348236492633,
        10445975478038012099,
        10499867969077387084,
        8910985952646879161,
        5688182352814120797,
        9274703596573355198,
        17079516738856648370,
        13154468210333061273,
        12035753711810209090,
        682457060097571089,
        18316833628521946911,
        5384603522444745788,
        7041952314487438562,
        1065320530637803495,
        18160547030144057620,
        1080268773617729602,
        16176995995382376864,
        12091324781952438262,
        3393646057006652951,
        12969809793558604459,
        2351120683106479909,
        427385498254739857,
        4663168181631519805,
        422970020340205891,
        12816720657824122729,
        17034481196571519911,
        3283192321028983485,
        10415653728094738100,
        4052504059936936456,
        10475299055785019719,
        17120352747060935870,
        5994910584295584054,
        15908275135755480183,
        12814428557221286596,
        14565382532028979499,
        16548162002930019948,
        3337313666594224013,
        6810438822289901006,
        2787876831199285079,
        17090579427497944842,
        7680145959990292652,
        12297168710949533327,
        15733055830098588352,
        8536975444014540557,
        3982197617899425809,
        2880644053685654332,
        3759595273251508317,
        9211592450055673635,
        2236732389336791955,
        379450029216573187,
        12222594382776322964,
        3426866009272788084,
    ],
    [
        6565822207575380947,
        7489336511789013212,
        2694189065366086672,
        16736955908384040667,
        11639392184357723841,
        15542901173920001049,
        3232453507984361786,
        4843551989393931806,
        18224856999204042062,
        18362210618352518334,
        4632255633194492386,
        17037666653945300038,
        857544086161213776,
        7831796323053974781,
        232428632632385146,
        2925088926737989508,
        3556355680467469208,
        8995118560757814484,
        5248467142624379973,
        11286938563189222715,
        5178285812603419988,
        10396326291342133054,
        14446580363543767093,
        1266056835947304905,
        12695527389209508464,
        2178204001882061487,
        17664729367612065620,
        2326858594448393611,
        7083477491439423569,
        243227827633583504,
        14713456511621626343,
        7217561773060103474,
        1787081505234355014,
        14223225102148286299,
        16356751535702812849,
        9742248055757414961,
        17213410736989765539,
        1234827922519094994,
        11651013493189809553,
        13912671995801873621,
        16275458560590129859,
        8503257859806096842,
        7898318417967638410,
        14151437813176201724,
        3174003374862698704,
        4064774330340655997,
        13369218493487265534,
        3351608848374700109,
        3252561897789668413,
        13810511708843904691,
        9269787387664705061,
        8839219903009012870,
        7107737956834339919,
        7549761418032178450,
        9339734576003656219,
        8004972211523223189,
        17346735961556837128,
        2505762905986433593,
        12566868148548889601,
        11696351332537226284,
        3719937566978825579,
        918440029480903188,
        14825684037400971261,
        5226380298206127654,
    ],
    [
        5011849201962620644,
        15551939760178152950,
        10482371124488929339,
        2313350342183900462,
        16820235977478832396,
        7666403286471046802,
        15945229576702828469,
        2264276905671112393,
        8483473686648681175,
        9985778038821338983,
        15437432140697142943,
        1795740586567723852,
        8521687418575192914,
        17261903142433401957,
        15204326651233853596,
        11405737787082608107,
        11414669708451398420,
        17895822829727618299,
        15752537011129755780,
        17582640925797961112,
        1605187692418176441,
        7002739385683260543,
        11673250062489790918,
        10024621371400181596,
        7549905477728820854,
        13996061569823867840,
        14045402851613937831,
        4211657577635167087,
        1870753691350548389,
        8748889289773835191,
        18015554452655095218,
        6718104849089448924,
        17764503863748009633,
        15574540811115377110,
        16494118714609188628,
        5737493910506582328,
        713557303720599794,
        2641133061062378869,
        11490263567753632848,
        17248470230417958147,
        15423354777644843516,
        7729063143455251612,
        2362746288148417389,
        7738279109856840130,
        2654902636418698060,
        18423254833995816428,
        18085840907194924907,
        754653202953696965,
        496244517705640953,
        17479439337585046589,
        10612122091984317938,
        6417573846889960707,
        8616424727110684666,
        12233306429535805268,
        5144990462888403322,
        5475528197433965124,
        8812543266680133867,
        14034602142729121456,
        1754157663109864268,
        1041530925011458728,
        10639253929270592775,
        3089156632675365165,
        15483760155896568682,
        14142342860013878638,
    ],
    [
        9487606973959112190,
        5159552436044472331,
        1058188405115403792,
        17589078293032447565,
        7113529570118427434,
        11375558468903029893,
        16333811356015620105,
        6581964997490704919,
        6163899598578516858,
        12808024782184896431,
        7231525416620510300,
        556288339211175775,
        7937194509539777389,
        3329429458097813412,
        8437165769926729209,
        3925943894493927455,
        4078694782157858723,
        1974373865149035587,
        8401347780128267458,
        7660133455053710339,
        8830955082715935345,
        321350232036802983,
        9778510739761232796,
        1182200591607817656,
        2431419123272158592,
        388096250593888615,
        15846603523668802683,
        4486373433022004640,
        5229645299018448805,
        16796561893438470424,
        6022934739434968487,
        8834097447358997941,
        6767319089758808317,
        15817365650579626553,
        11548285625767207543,
        15907231329118662008,
        8308500307476354003,
        15427496437531838876,
        2820296602550479969,
        14002089021466706984,
        8062069094431701482,
        652274764174689194,
        16825016147266188247,
        9811093685850926365,
        6142809394873311978,
        15191326982067098465,
        15134705187902919777,
        6498481264526518584,
        874602723952506184,
        8165034629699358461,
        9627914091396991265,
        17737291027626001700,
        15054045613328422175,
        3197567908748462743,
        6012303867310924140,
        9609921641222585382,
        8676504749500996425,
        5247817425181311158,
        6238747609746216883,
        12040217243058155462,
        17723112942660558875,
        14398729334225899188,
        6133171340696412448,
        11701824201630722116,
    ],
    [
        8586501732831427874,
        5885104241129745510,
        8491494132852451616,
        12976349108160977172,
        6295966558130134398,
        11266501897529250852,
        16518221805849930519,
        7837264892598308534,
        14562365718000236820,
        17957182162939509789,
        6075784181855816582,
        16188812807282538144,
        908635247764705999,
        1156583086659749943,
        13996205361670808455,
        8514459597709537063,
        11021560709760596614,
        2067989397114811537,
        12879470862810696425,
        16162037568963272235,
        5562490149000279383,
        6335094431359433324,
        15380474722875132941,
        3522526184914226340,
        13803096132941381765,
        14000678089194127906,
        12965882401616450037,
        7263568225941161097,
        4753526830019978789,
        15750724394912267505,
        11895617787179193336,
        9942882010649402973,
        13721796179721299319,
        3753561347829429012,
        3152358305110635449,
        1063101041703033197,
        16801651265125195774,
        11907076481255889151,
        15686758739336770364,
        14513201606396142690,
        11211613521031119048,
        8170498565652629937,
        2589644629254668021,
        10868558896161349581,
        8446131224759201924,
        12545117201749468720,
        10423338540592673500,
        2197291980894421288,
        15419100967983303185,
        5071417891293942790,
        3502469209381547926,
        15121133302982587408,
        13877979451713530317,
        17825429662726222068,
        16668360647317635391,
        9022944923891668116,
        229248291721353609,
        5236829256797153830,
        8882007624818859893,
        11538577823746897897,
        11776661920651998536,
        5291108703106140703,
        16350092163378817453,
        1063479632041383297,
    ],
    [
        7818515129474820611,
        3223095482874274351,
        10073878031669758835,
        4506765874329868621,
        9685023455557375838,
        9388288988946164686,
        6760591939578164061,
        12077905529369799551,
        3749549724118220036,
        9072285927808983439,
        997245558680087416,
        1993662627682197717,
        16784957320030881774,
        6822821411057387165,
        18073054900150491765,
        12577481712844021104,
        6059183127970527645,
        16887101961767630527,
        1275360422421367873,
        14006329855974485012,
        14095165788412216974,
        13353468166353801627,
        3740081413756930353,
        4117988352417495347,
        10894990238394827899,
        14452509693389043047,
        13056890138310196354,
        1448415105540034643,
        1862272727906873976,
        7440353020029876483,
        12944453550275051111,
        1178171428292784468,
        17266307346320296596,
        14638493868304375026,
        5043988453325561600,
        1945762859027100657,
        11392504191751720798,
        12413791483281451444,
        10878785020410323286,
        16315223671816537758,
        4979145312974494169,
        4641785662962223703,
        7161559668185993191,
        9903302310245773759,
        17879873826661205865,
        6634489287089509447,
        10746394200407320306,
        2299874420822964658,
        18066691454744997436,
        5046706864134289830,
        6074958995069832481,
        6710130046949698617,
        10525286905260248511,
        4574575762550418374,
        18179770629004864708,
        4765473365782964582,
        4503984569453031386,
        6958689760628660793,
        6501632046901395292,
        4952810891235371962,
        5637015941430686461,
        4689899239780205083,
        6227595751193559286,
        4584303520771138819,
    ],
    [
        5818548507619157503,
        1913016423686850180,
        5158634071355515588,
        10974221046589205985,
        6027408195213618866,
        2861097681853867942,
        12307334860639502101,
        7618525510835310400,
        10389695774697314487,
        676281819191486403,
        3714691713780769423,
        6208093757943897884,
        6524645644238534426,
        802246485013762431,
        5268740453222878329,
        10171021412979247819,
        4238995991084486577,
        8947460391469175654,
        16957744828235969086,
        13311667034111225089,
        1434035641456817927,
        8264986892402658000,
        1344980286869759189,
        3813755767814398632,
        10879016405042921359,
        13247245981237998969,
        13189344016036144052,
        9460077862199954080,
        13318351870875383548,
        9117002201178048841,
        16805223203120972131,
        14899176559084921564,
        14231007409368808716,
        10321456851059225626,
        4169911419901074111,
        6370505606265396521,
        2765500370942201777,
        6033212418423568860,
        15621841555676445198,
        16893138401786261949,
        11189364690250869810,
        523778459764899667,
        4493032449662546700,
        13707677326560958092,
        7096085405504875686,
        9600326813443278128,
        6505025200724020066,
        13679496933829510547,
        6319526175891444977,
        12341976107773932715,
        6305975344048003589,
        466375982206659890,
        3179822338548320926,
        18101051059120358744,
        1240531694745350220,
        9348645157028658403,
        2568896355961394693,
        2737803908572245141,
        7658505107422437313,
        7274153728058279940,
        7331735845404663269,
        6527709815692606614,
        8052044244347557366,
        11742764566849525460,
    ],
];
