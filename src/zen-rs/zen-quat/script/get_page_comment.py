import pymupdf
import sys

name = sys.argv[1]
doc = pymupdf.open(name)

select = []
for page in doc:
    exist = next(page.annots(), None) is not None
    if exist:
        select.append(page.number)

print(select)
doc.select(select)
doc.save(name[:-4] + "_extract.pdf")
