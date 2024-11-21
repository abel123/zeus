import fitz
import pymupdf


def excute(filename, list):
    doc = pymupdf.open(filename)
    with open(list, "r") as f:
        data = f.readlines()
        toc = [
            [1, "  -  ".join(l.rsplit(maxsplit=1)[0].split(maxsplit=1)), idx + 1]
            for idx, l in enumerate(data[1:])
        ]
        doc.set_toc(toc)
        doc.save(filename, incremental=True, encryption=fitz.PDF_ENCRYPT_KEEP)
