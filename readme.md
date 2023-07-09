# moxdtrkr
_This is a side-project. It only sees development when I have free time, which is not that often._
## About

moxdtrkr (said "mocks-tracker") is a tool to aid in managing personal finances. 

Initially (and currently) it only provides tools to track expenditure - integrated budgeting tools will be implemented in the future (date unspecified/unknown).

It is currently CLI-only. It has been under development since October 2022 in a different repository. In June 2023 it was open-sourced and moved to this repository once the name was decided upon and the software was somewhat usable by anyone other than the developer.

The name is derived from a mash-up of the letters from "MOney OXiDe TRacKeR", where "OxideTracker" was the temporary name assigned to the project from day -1, and it a pun on the Rust language. The name also had to not be in use elsewhere

## Security
VERY IMPORTANT: moxdtrkr stores all data in json files. This effectively means that all the information you ever input into the software has the potential to be stored in plaintext.

All information is stored locally on the device in which it runs. This means the user has total control ( *and responsibility* ) for their own personal data, which by nature of the product is highly sensitive.

The user is responsible for performing a risk assessment, and any mitegation if they decide it necessary, with regards to storing this information in plaintext. The developer can accept absolutely no fault or liability with regards to any data.

## Licence & costs
This software is distributed under GPL.

Whilst it is free (in both senses), if you benefit greatly from this software, please consider donating if donations are accepted in the future. Currently, donations are not accepted.

# Downloads
TODO

# Version History & Changelogs

## _Version 2.0_ CURRENTLY IN DEVELOPMENT - CHANGELOG DUE TO CHANGE
### Feature Updates (graphing and UX improvements)
- Summary bar graph added - shows weekly expenditure by choice of account(s) and category(s)
- 'List Categories' functionality added
- Removed superfluous outputs and made the UI design more consistent
- Added settings menu so program parameters can be adjusted easily
- Option to save without exiting
 
### Bugfixes
- Consistent weeks starting on Monday (rather than a mix of Sunday and Monday)


## _Version 1.1_
### Bugfixes
- Removed rounding error from amount input calculation (e.g. inputting 157.7 now gives £157.70 rather than the £157.69 it gave before)
- Wrap text correctly in day pane in calendar
- Erroneous highlighting of day in calendar when in sub-menus removed
- Exiting 'new transaction based on current' menu no longer resets selection in day sub-menu
- Fixed padding in ListAccounts for long account names

## _Version 1.0_
### Installation
  Create the following directory structure
- ```OxideTracker```  (Directory; any sensible name)
    - ```data``` (Directory)
      - The save data of the program will be generated and stored here in json files
    - ```OxideTracker``` (Executable)

Run the program, see that it errors and starts new accounts and transactions databases, then exit the program and check both ```data/accounts.json``` and ```data/transactions.json``` have been generated.
Run the program again, and there should not be any more error messages, and it should be installed and ready to use.
### Features
- Accounts
- Transactions
- Transfers
- Transaction categories
- Calendar view
- Detailed transaction view for a given day incl. closing balance at the end of that day
- Modify and Delete Transactions and Transfers
- Create a new Transaction or Transfer based on an existing one
- Show all Accounts and their current balance
- Choose which account(s) to show in the Calendar View (all accounts by default)
- Transactions and Accounts stored on program exit and read in on program start
