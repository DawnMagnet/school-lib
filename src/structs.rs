#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestWithCookiesPageList {
    pub id: String,
    pub isdel: bool,
    pub uid: String,
    pub rname: String,
    pub address: String,
    pub rtype: String,
    pub allowuser: i64,
    pub minuser: i64,
    pub venceid: String,
    pub gateid: String,
    pub status: bool,
    pub opentime: bool,
    pub appointnum: i64,
    pub rulepoliy: String,
    pub selected: bool,
    pub authid: String,
    pub gatestatus: i64,
    pub opensign: bool,
    pub isseat: bool,
    pub payrule: String,
    pub rno: i64,
    pub pics: String,
    pub wfId: String,
    pub checkuser: bool,
    pub times: Vec<RequestWithCookiesTimes>,
    pub timeSpace: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestWithCookiesParams {
    pub rooms: RequestWithCookiesRooms,
    pub ruleId: String,
    pub usetype: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestWithCookiesRooms {
    pub currentPage: i64,
    pub showPage: Vec<i64>,
    pub nextPage: i64,
    pub next: bool,
    pub front: bool,
    pub frontPage: i64,
    pub pageSize: i64,
    pub totalCount: i64,
    pub totalPage: i64,
    pub pageList: Vec<RequestWithCookiesPageList>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestWithCookiesRootInterface {
    pub status: bool,
    pub content: String,
    pub params: RequestWithCookiesParams,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestWithCookiesTimes {
    pub select: bool,
    pub time: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersonConfig {
    pub config_name: String,
    pub para: String,
    pub isFlag: String,
    pub openid: String,
    pub account: String,
    pub upass: String,
    pub headimgurl: String,
    pub nickname: String,
    pub sign: String,
    pub timestamp: String,
    pub params: String,
    pub roomId: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BookStoreInfoConfig {
    pub OPEN_ID: String,
    pub JSESSIONID: String,
    pub X_CSRF_TOKEN: String,
    pub UUID: String,
    pub RULE_ID: String,
    pub PREFER: String,
    pub PERSON: Vec<PersonConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppointMyappionts {
    pub currentPage: i64,
    pub showPage: Vec<i64>,
    pub nextPage: i64,
    pub next: bool,
    pub front: bool,
    pub frontPage: i64,
    pub pageSize: i64,
    pub totalCount: i64,
    pub totalPage: i64,
    pub pageList: Vec<AppointPageList>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppointPageList {
    pub id: String,
    pub stime: String,
    pub userid: String,
    pub uid: String,
    pub ctime: String,
    pub pay: i64,
    pub sign: bool,
    pub etime: String,
    pub currentday: String,
    pub signtime: String,
    pub rname: String,
    pub status: i64,
    pub flag: i64,
    pub title: Option<String>,
    pub bstatus: Option<i64>,
    pub cstatus: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppointParams {
    pub myappionts: AppointMyappionts,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppointRootInterface {
    pub status: bool,
    pub content: String,
    pub params: AppointParams,
}
#[derive(Debug, Clone)]
pub struct SeatInfo {
    pub id: String,
    pub rname: String,
    pub times: String,
    pub avai: u32
}

#[derive(Debug, Clone)]
pub struct AppointmentInfo {
    pub id: String,
    pub begin_time: String,
    pub end_time: String,
    pub rname: String,
    pub status: i32,
    pub flag: i32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppointNode {
    pub _stime: String,
    pub _etime: String,
    pub _roomid: String,
    pub _currentday: String,
    pub UUID: String,
    pub ruleId: String,
    pub users: String,
    pub usercount: String,
    pub room_exp: String,
    pub _seatno: String,
    pub LOCK: String,
    pub _uid: String,
    pub _status: i64,
    pub _id: Option<String>,
    pub _userid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppointmentParams {
    pub node: AppointNode,
    pub _id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppointmentRootInterface {
    pub status: bool,
    pub content: String,
    pub params: AppointmentParams,
}
