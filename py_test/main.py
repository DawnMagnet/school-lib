import os
from apscheduler.schedulers.background import BackgroundScheduler
from school_library import BookStoreInfo

scheduler = BackgroundScheduler(timezone='Asia/Shanghai')

bi = BookStoreInfo("config.toml")


def now_time_pd():
    from pandas import to_datetime
    from datetime import datetime
    return to_datetime(datetime.now())


def cur_time_str():
    import time
    return time.strftime("%Y-%m-%d %H:%M:%S", time.localtime())


def make_new_line():
    print("> ", end='')


@scheduler.scheduled_job('cron', minute='5, 30', hour='7-22', id="refresh", max_instances=100)
def refresh():
    bi.__init__("config.toml")
    print(cur_time_str(), "config1", bi.sign('config1'))
    print(cur_time_str(), "config2", bi.sign('config2'))
    make_new_line()


@scheduler.scheduled_job('cron', hour='0', minute='0', second='0', id='nxt_day_app')
def scheduled_appointment(seat=None):
    bi.__init__("config.toml")
    res = bi.make_one_seat_every_appointment(room_id=seat, force=True)
    print("[SCHEDULED RESULT]")
    for a, (b, c) in res:
        print(f'{str(a):<20}{b}\t{c}')


scheduler.start()

if __name__ == "__main__":
    print('WelCome to SCL REPL v0.5!\nPrint "help" for more information\n> ', end='')
    while True:
        try:
            command = input().strip().split()
            if not command:
                pass
            elif command[0] == "help":
                print("""
    [{}]
    help: print this help message
    exit: exit the program
    la  : print school library full list
    ls  : print school library available list
    ap  : print appointments list
    jb  : print background jobs(debug)
    r   : force refresh info(debug)
    cs  : cancel schedule
    sg  : sign(force)
    st  : set current seat
    sn  : schedule now(force)
    s   : schedule next_day
                """.format(cur_time_str()))
            elif command[0] == "exit":
                scheduler.shutdown(wait=False)
                print("Bye!")
                break
            elif command[0] == "la":
                bi.show_full_data()
            elif command[0] == "ls":
                bi.show_available_data()
            elif command[0] == "ap":
                bi.show_appointment_to_be_signed()
            elif command[0] == "raw_ap":
                bi.show_raw_appointment()
                # dprint(bi.raw_appointment)
            elif command[0] == "remove_ap":
                print(bi.cancelAppointment(command[1]).text)
            elif command[0] == "r":
                refresh()
                print('\b\b', end='')
            elif command[0] == "jb":
                scheduler.print_jobs()
            elif command[0] == "sg":
                if len(command) > 1:
                    print(bi.sign(command[1]))
                else:
                    print(bi.sign())
            elif command[0] == "s":
                from datetime import datetime

                app_time = datetime.now()
                app_time = app_time.replace(
                    day=app_time.day + 1, hour=0, minute=0, second=2, microsecond=0)
                # app_time = app_time.replace(second=app_time.second + 1)
                print("Job Will Start At {}.".format(app_time))
                scheduler.add_job(scheduled_appointment, 'date',
                                  run_date=app_time, id='nxt_day_app')
            elif command[0] == "sn":
                if len(command) > 1:
                    scheduled_appointment(command[1])
                else:
                    scheduled_appointment()
            elif command[0] == "cs":
                scheduler.remove_job("nxt_day_app")
            elif command[0] == "clear":
                os.system('clear')
            else:
                print('Unknown command\nPrint "help" for more information')
            make_new_line()
        except Exception as e:
            print(e)
            make_new_line()
            pass
