CREATE TABLE IF NOT EXISTS "spis"
(
    "id" INTEGER PRIMARY KEY AUTOINCREMENT,
    "numerKRS" TEXT NOT NULL,
    "formaPrawna" TEXT,
    "regon" TEXT,
    "nip" TEXT,
    "nazwa" TEXT NOT NULL,
    "kraj" TEXT,
    "wojewodztwo" TEXT,
    "powiat" TEXT,
    "gmina" TEXT,
    "miejscowosc" TEXT,
    "ulica" TEXT,
    "nrDomu" TEXT,
    "kodPocztowy" TEXT,
    "poczta" TEXT,
    "adresDoDoreczenElektronicznychWpisanyDoBAE" TEXT
);

-- INSERT INTO "spis"
-- VALUES(1, "0000000", "", "00000000", "000000000", "ABCDEFGH", "Polska", "Pomorskie", "gdański", "gdańsk", "Gdańsk", "Polna", "1", "80-666", "Gdańsk");