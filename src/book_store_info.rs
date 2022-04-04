use std::fmt::Debug;
use std::time::Duration;
use pyo3::prelude::*;

use chrono::{Local, NaiveDateTime};
use pyo3::exceptions::PyRuntimeError;
use regex::Regex;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::redirect;
use reqwest::header::HeaderMap;

use crate::structs::{AppointmentInfo, AppointmentRootInterface, AppointRootInterface, BookStoreInfoConfig, RequestWithCookiesPageList, RequestWithCookiesRootInterface, SeatInfo};
type MyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const COMMON_HEADERS: [(&str, &str); 7] = [
    ("Connection", "keep-alive"),
    ("DNT", "1"),
    ("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.93 Safari/537.36"),
    ("Upgrade-Insecure-Requests", "1"),
    ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"),
    ("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6"),
    ("Cache-Control", "max-age=0"),
];

const SEC_LIST: [&str; 5] = ["", "0a4c97c5b7844420abdc7128715b8885",
    "", "", "31df48baed5148a5ae4eb219cdd1e415"];

fn tr(add_on: &[(&'static str, &str)], default: bool) -> HeaderMap {
    let mut res = HeaderMap::new();
    if default {
        for (k, v) in &COMMON_HEADERS {
            res.insert(*k, v.parse().unwrap());
        }
    }
    for (k, v) in add_on {
        res.insert(*k, v.parse().unwrap());
    }
    res
}
fn deal_rname(s: &str) -> String {
    let vec_ch: Vec<char> = s.chars().collect();
    if vec_ch.len() != 13 {
        "InValid RoomID".to_string()
    } else {
        format!("{}-{}-{}{}", vec_ch[2].to_string().into_bytes()[2] - 159, vec_ch[8], vec_ch[10], vec_ch[11])
    }
}

#[derive(Debug)]
#[pyclass]
pub struct BookStoreInfo {
    pub config_path: String,
    pub config: BookStoreInfoConfig,
    pub raw_data: Vec<SeatInfo>,
    pub full_data: Vec<SeatInfo>,
    pub available_data: Vec<SeatInfo>,
    pub raw_appointment: Vec<AppointmentInfo>,
    pub appointment_to_be_signed: Vec<AppointmentInfo>,
}

impl BookStoreInfo {
    pub fn new(config_path: &str) -> MyResult<Self> {
        let config_str = std::fs::read_to_string(config_path)?;
        let mut res = BookStoreInfo {
            config_path: config_path.to_string(),
            config: toml::from_str(&config_str)?,
            raw_data: Default::default(),
            full_data: Default::default(),
            available_data: Default::default(),
            raw_appointment: Default::default(),
            appointment_to_be_signed: Default::default(),
        };
        res.refresh_available_info()?;
        res.raw_get_appointment_records()?;
        MyResult::Ok(res)
    }
    pub fn refresh(&mut self) -> MyResult<()> {
        self.refresh_available_info()?;
        self.raw_get_appointment_records()?;
        // println!("REFRESH INFO DONE!");
        Ok(())
    }
    pub  fn get_origin_info(&mut self, sec: usize) -> MyResult<Vec<SeatInfo>> {
        let mut result = self.raw_request_with_cookies(sec);
        if result.is_err() {
            let (jsessionid, x_csrf_token) = self.raw_get_new_cookies()?;
            self.config.JSESSIONID = jsessionid;
            self.config.X_CSRF_TOKEN = x_csrf_token;
            self.write_toml();
            result = self.raw_request_with_cookies(sec);
        }
        let (data, rule_id) = result?;
        self.config.RULE_ID = rule_id;
        self.write_toml();
        Ok(data.iter()
            .map(|d| {
                let mut count = 0;
                let time: String = d.times.iter().map(|x| {
                    if x.select {
                        'X'
                    } else {
                        count += 1;
                        'O'
                    }
                }).collect();
                SeatInfo {
                    id: d.id.clone(),
                    rname: d.rname.clone(),
                    times: time,
                    avai: count,
                }
            }).collect())
    }
     fn raw_request_with_cookies(&mut self, sec: usize) -> MyResult<(Vec<RequestWithCookiesPageList>, String)> {
        let section = SEC_LIST[sec];
        let unknown = "57879bf578f24a43bae98434682bf176";
        let day_str = Local::now().format("%Y-%m-%d").to_string();
        let url = format!("http://libwx.cau.edu.cn/space/discuss/findRoom/{}/{}/{}", section, unknown, day_str);
        let headermap = tr(&[
            ("X-CSRF-TOKEN", &self.config.X_CSRF_TOKEN),
            ("Cookie", &format!("JSESSIONID={}", self.config.JSESSIONID)),
            ("Content-Type", "application/x-www-form-urlencoded")
        ], false);
        let res = Client::new()
            .post(url)
            .headers(headermap)
            .body("currentPage=1&pageSize=100")
            .send()?
            .json::<RequestWithCookiesRootInterface>()?;
        let params = res.params;
        Ok((params.rooms.pageList, params.ruleId))
    }
     fn raw_get_new_cookies(&mut self) -> MyResult<(String, String)> {
        let headers = tr(&[], true);
        let url = format!("http://libwx.cau.edu.cn/remote/static/authIndex?parameter=1&openid={}", self.config.OPEN_ID);
        let res = Client::new()
            .get(&url)
            .headers(headers)
            .timeout(Duration::from_secs(3))
            .send()?
            .text()?;
        let url_suffix = Regex::new("window.location.href = urls \\+ \"(.*)\";")?
            .captures(&res).unwrap()[0].to_string();
        let headers = tr(
            &[("Referer", "http://libwx.cau.edu.cn/remote/static/authIndex?parameter=1&openid=oJ7t-1fCfr-FokhmYcI5QerAJIxo")],
            true,
        );
        let prevent_redirect_policy = redirect::Policy::custom(|attempt| { attempt.stop() });
        let res = ClientBuilder::new()
            .redirect(prevent_redirect_policy)
            .build()?
            .get(format!("http://libwx.cau.edu.cn/space/static/dowechatlogin?type=discuss{}", url_suffix))
            .headers(headers)
            .send()?;
        let jsessionid = res.headers().get("Set-Cookie").unwrap().to_str()?;
        let jsessionid = &jsessionid[11..43];
        let headers = tr(&[
            ("Cookie", &format!("JSESSIONID={}", jsessionid)),
            ("Referer", "http://libwx.cau.edu.cn/space/discuss/notice?linkSign=notice&type=discuss&noticeId=7f35dde178074b17bc547ba78160930c")
        ], true);
        let res = Client::new()
            .get("http://libwx.cau.edu.cn/space/discuss/mobileIndex?linkSign=discuss&type=discuss")
            .headers(headers)
            .send()?
            .text()?;
        let x_csrf_token = Regex::new("name=\"_csrf\" content=\"(.*)\"")?
            .captures(&res).unwrap()[1].to_string();
        Ok((jsessionid.to_string(), x_csrf_token))
    }

    pub  fn refresh_available_info(&mut self) -> MyResult<()> {
        let res_1 = self.get_origin_info(1)?;
        let res_4 = self.get_origin_info(4)?;
        self.raw_data = [res_1, res_4].concat();
        self.full_data = self.deal_raw_data(false);
        self.available_data = self.deal_raw_data(true);
        Ok(())
    }
     fn raw_get_appointment_records(&mut self) -> MyResult<Vec<AppointmentInfo>> {
        let headers = tr(&[
            ("Cookie", &format!("JSESSIONID={}", self.config.JSESSIONID)),
            ("Referer", "http://libwx.cau.edu.cn/space/discuss/myAppoint?linkSign=myReserve&type=discuss")
        ], true);
        let res = Client::new()
            .get("http://libwx.cau.edu.cn/space/discuss/queryAppiont?cday=1970-01-01_to_2050-01-01&sign=&rtypeid=&type=discuss")
            .headers(headers)
            .send()?
            .json::<AppointRootInterface>()?;
        let v_app = res.params.myappionts.pageList.iter()
            .filter(|x| !x.sign)
            .filter(|x| {
                let begin_time = format!("{} {}", x.currentday, x.stime);
                if let Ok(app_time) = NaiveDateTime::parse_from_str(&begin_time, "%Y-%m-%d %H:%M") {
                    let now = Local::now().naive_local();
                    now < app_time
                } else {
                    println!("ERROR FORMAT: {}", begin_time);
                    true
                }
            }).cloned()
            .collect::<Vec<_>>();
        let res = v_app
            .iter()
            .map(|x| {
                AppointmentInfo {
                    id: x.id.clone(),
                    begin_time: format!("{} {}", x.currentday, x.stime),
                    end_time: x.etime.clone(),
                    rname: x.rname.clone(),
                    status: x.status as i32,
                    flag: x.flag as i32,
                }
            }).collect::<Vec<_>>();
        self.appointment_to_be_signed = res.clone();
        Ok(res)
    }

     fn raw_make_one_appointment(&self, room_id: &str, start_hour: i32, remain_hours: i32) -> MyResult<AppointmentRootInterface> {
        let now = Local::now();
        let now_str: String = now.to_rfc3339();
        let today = &now_str[0..10];
        let begin_time = start_hour * 60;
        let end_time = (start_hour + remain_hours) * 60;
        let rule_id: &str = &self.config.RULE_ID;
        let header = tr(&[
            ("Content-Type", "application/json"),
            ("X-CSRF-TOKEN", &self.config.X_CSRF_TOKEN),
            ("X-Requested-With", "XMLHttpRequest"),
            ("Origin", "http://libwx.cau.edu.cn"),
            ("Referer", &format!("http://libwx.cau.edu.cn/space/discuss/openAppointDetail?roomid={}&ustime={}&uetime={}&selectDate={}&ruleId={}&mobile=true&linkSign=discuss", room_id, begin_time, end_time, today, rule_id)),
            ("Pragma", "no-cache"),
            ("Cache-Control", "no-cache"),
            ("Cookie", &format!("JSESSIONID={}", self.config.JSESSIONID))
        ], true);
        let data = format!(r##"{{"_stime": "{}", "_etime": "{}", "_roomid": "{}", "_currentday": "{}", "UUID": "VEmkgCYM", "ruleId": "{}", "users": "2019307070109 2019321010102", "usercount": "2", "room_exp": "[]", "_seatno": "0", "LOCK": "true"}}"##, begin_time, end_time, room_id, today, rule_id);

        Ok(Client::new()
            .post("http://libwx.cau.edu.cn/space/form/dynamic/saveFormLock")
            .headers(header)
            .body(data)
            .send()?
            // .text()?
            .json::<AppointmentRootInterface>()?
        )
    }
    pub  fn make_one_seat_every_appointment(&self, room_id: Option<&str>, force: Option<bool>) -> MyResult<Vec<(Vec<i32>, (String, String))>> {
        let real_room_id = room_id.unwrap_or(&self.config.PREFER);
        let real_force = force.unwrap_or(false);
        let available_period = if real_force {
            vec![vec![20, 21], vec![17, 18, 19],
                 vec![14, 15, 16], vec![11, 12, 13], vec![8, 9, 10]]
        } else {
            let mut tmp: Vec<Vec<i32>> = vec![];
            let (mut tmp_len, mut lst_len) = (0, 0);
            let time_period = self.full_data.iter().find(|x| x.id == real_room_id).ok_or("NO SUCH ROOM")?.times.clone().into_bytes();
            for hour in 8..22 {
                if time_period[hour as usize - 8] == b'O' {
                    if tmp_len > 0 && lst_len < 3 && tmp[tmp_len - 1][lst_len - 1] as i32 == hour - 1 {
                        tmp[tmp_len - 1].push(hour);
                        lst_len += 1;
                    } else {
                        tmp.push(vec![hour]);
                        tmp_len += 1;
                        lst_len = 1;
                    }
                }
            }
            tmp
        };
        let res_tmp = available_period.iter().map(
            |available_time_period|  {
                match self.raw_make_one_appointment(real_room_id, available_time_period[0], available_time_period.len() as i32) {
                    Ok(appoint) => (appoint.status.to_string(), appoint.content),
                    Err(e) => ("error".to_string(), e.to_string())
                }
            }
        );
        let mut res = vec![];
        for (a, b) in res_tmp.into_iter().enumerate() {
            res.push((available_period[a].clone(), b));
        }
        Ok(res)
    }
    // pub  fn cancel_appointment(&mut self, ) -> MyResult<Response> {}
    pub fn raw_sign(&mut self, sign_config: Option<String>, room_id: Option<String>) -> MyResult<String> {
        let real_sign_config = sign_config.unwrap_or_else(|| "config1".to_string());
        let real_room_id = room_id.unwrap_or({
            if self.appointment_to_be_signed.is_empty() {
                return Ok("No Appoint at that time!".to_string());
            }
            let room_name: &str = &self.appointment_to_be_signed.iter().nth_back(0).ok_or("WON'T WRONG")?.rname;
            self.full_data.iter().find(|x| x.rname == room_name).ok_or("NO SUCH ROOM")?.id.clone()
        });
        let headers = tr(&[], true);
        let mut params = self
            .config
            .PERSON
            .iter().find(|x| x.config_name == real_sign_config)
            .ok_or("NO SUCH CONFIG")?
            .clone();
        params.roomId = Some(real_room_id);
        let res = Client::new()
            .get("http://libwx.cau.edu.cn/space/static/cau/mediaCheckIn")
            .headers(headers)
            .query(&params)
            .send()?
            .text()?;
        let res = &Regex::new("<span>(.*)</span>")?
            .captures(&res).unwrap()[1];
        match res {
            "当前预约已签到" => Ok("Already!".to_string()),
            "系统仅允许提前60分钟签到！" => Ok("Not Reach Time".to_string()),
            x => Ok(x.to_string())
        }
    }


    fn write_toml(&mut self) {
        let config_str = toml::to_string(&self.config).unwrap();
        std::fs::write(&self.config_path, config_str).unwrap();
    }
    fn deal_raw_data(&mut self, available_only: bool) -> Vec<SeatInfo> {
        let mut res = if available_only {
            self.raw_data.iter().filter(|x| x.avai > 0).cloned().collect::<Vec<_>>()
        } else {
            self.raw_data.clone()
        };
        res.sort_unstable_by_key(|x| x.rname.clone());
        res.reverse();
        res.sort_by_key(|x| x.avai);
        res.reverse();
        res
    }
    pub fn show_seat_info(v: &[SeatInfo]) {
        if !v.is_empty() {
            println!("               id                rname      times      A");
            for seat in v {
                println!("{} {} {} {}", &seat.id, deal_rname(&seat.rname), &seat.times, &seat.avai);
            }
        } else {
            println!("[INFO] SEAT LIST NO DATA")
        }
    }
    pub fn show_appointment_info(v: &[AppointmentInfo]) {
        if !v.is_empty() {
            println!("               id                   date    begin  end  rname  F S");
            for appoint in v {
                println!("{} {} {} {} {} {}", &appoint.id, &appoint.begin_time, &appoint.end_time, deal_rname(&appoint.rname), &appoint.flag, &appoint.status);
            }
        } else {
            println!("[INFO] SEAT LIST NO DATA")
        }
    }
}

#[pymethods]
/// Export Python Methods
impl BookStoreInfo {
    #[new]
    pub fn py_new(config_path: &str) -> PyResult<Self> {
        BookStoreInfo::new(config_path).or(Err(PyRuntimeError::new_err("Initialize Unsuccessful")))
    }
    #[pyo3(name = "sign")]
    pub fn py_sign(&mut self, sign_config: Option<String>, room_id: Option<String>) -> PyResult<String> {
        self.raw_sign(sign_config, room_id).or(Err(PyRuntimeError::new_err("Sign Unsuccessful")))
    }
    #[pyo3(name = "make_one_seat_every_appointment")]
    pub fn py_make_one_seat_every_appointment(&self, room_id: Option<&str>, force: Option<bool>) -> PyResult<Vec<(Vec<i32>, (String, String))>> {
        self.make_one_seat_every_appointment(room_id, force).or(Err(PyRuntimeError::new_err("Unable to make one seat every appointment")))
    }
    #[pyo3(name = "show_full_data")]
    pub fn py_show_full_data(&self) {
        Self::show_seat_info(&self.full_data);
    }
    #[pyo3(name = "show_raw_data")]
    pub fn py_show_raw_data(&self) {
        Self::show_seat_info(&self.raw_data);
    }
    #[pyo3(name = "show_available_data")]
    pub fn py_show_available_data(&self) {
        Self::show_seat_info(&self.available_data);
    }
    #[pyo3(name = "show_raw_appointment")]
    pub fn py_show_raw_appointment(&self) {
        Self::show_appointment_info(&self.raw_appointment);
    }
    #[pyo3(name = "show_appointment_to_be_signed")]
    pub fn py_show_appointment_to_be_signed(&self) {
        Self::show_appointment_info(&self.appointment_to_be_signed);
    }
    #[pyo3(name = "refresh")]
    pub fn py_refresh(&mut self) -> PyResult<()> {
        self.refresh().or(Err(PyRuntimeError::new_err("Refresh Unsuccessful")))
    }
}