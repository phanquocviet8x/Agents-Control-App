import gspread
from google.oauth2.service_account import Credentials
SCOPES = ['https://www.googleapis.com/auth/spreadsheets']
creds = Credentials.from_service_account_file(r'C:\Users\caxin\Desktop\account-google.json', scopes=SCOPES)
gc = gspread.authorize(creds)
sh = gc.open_by_key('1n5mSJOJ5jno5FZ27zTibjXTFwdY68F47EiO8aAeZw-A')
ws = sh.worksheet('App_Actions')
rows = ws.get_all_values()
for i, r in enumerate(rows):
    print(f'{i}: {r}')
