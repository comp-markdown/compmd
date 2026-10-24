---
md:
  inputs:
    since: date = (md.today())
    region: all | eu | us = all
    query: string = ""
    on: boolean = false
---

:: date-input[Since] {bind=since}
:: select-input[Region] {bind=region}
:: text-input[Query] {bind=query}
:: toggle-input[On] {bind=on}

:: date-input[Wrong type] {bind=region}
:: select-input[Not an enum] {bind=query}
:: number-input[Undeclared] {bind=count}
