-- =====================================================================
-- rvhub - Modele physique de donnees (SQLite)
-- Collecteur de compilations pour les projets rvkit
-- Methode de conception : Merise (MCD -> MLD -> MPD), 3e forme normale
-- =====================================================================

PRAGMA foreign_keys = ON;

-- ---------------------------------------------------------------------
-- Table CARTE : referentiel des cibles materielles supportees
-- RG1 : le nom d'une carte est unique
-- RG2 : l'architecture est contrainte a un ensemble ferme de valeurs
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS carte (
    id_carte    INTEGER PRIMARY KEY AUTOINCREMENT,
    nom         TEXT    NOT NULL UNIQUE,
    cpu_arch    TEXT    NOT NULL CHECK (cpu_arch IN ('riscv32')),
    flash_tool  TEXT    NOT NULL CHECK (flash_tool IN ('wlink', 'esptool')),
    flash_ko    INTEGER NOT NULL CHECK (flash_ko > 0),
    ram_ko      INTEGER NOT NULL CHECK (ram_ko > 0)
);

-- ---------------------------------------------------------------------
-- Table COMPILATION : historique des builds
-- RG3 : une compilation porte sur une carte existante (integrite ref.)
-- RG4 : la taille produite ne peut etre negative
-- RG5 : la taille ne peut depasser la flash disponible sur la carte
-- RG6 : le statut appartient a un ensemble ferme
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS compilation (
    id_compilation INTEGER PRIMARY KEY AUTOINCREMENT,
    id_carte       INTEGER NOT NULL REFERENCES carte(id_carte) ON DELETE CASCADE,
    projet         TEXT    NOT NULL,
    taille_octets  INTEGER NOT NULL CHECK (taille_octets >= 0),
    statut         TEXT    NOT NULL CHECK (statut IN ('succes', 'echec')),
    compile_le     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_compilation_carte_date
    ON compilation(id_carte, compile_le);

-- ---------------------------------------------------------------------
-- Table JOURNAL : tracabilite des operations sensibles
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS journal (
    id_journal  INTEGER PRIMARY KEY AUTOINCREMENT,
    entite      TEXT NOT NULL,
    id_entite   INTEGER,
    operation   TEXT NOT NULL,
    detail      TEXT,
    effectue_le TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ---------------------------------------------------------------------
-- DECLENCHEUR 1 : refus applicatif d'un binaire depassant la flash
-- L'integrite est garantie PAR LA BASE : aucun client ne peut la
-- contourner, meme en ecrivant directement en SQL.
-- ---------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS trg_verifie_taille_flash
BEFORE INSERT ON compilation
FOR EACH ROW
WHEN NEW.statut = 'succes'
BEGIN
    SELECT CASE
        WHEN NEW.taille_octets > (SELECT flash_ko * 1024 FROM carte WHERE id_carte = NEW.id_carte)
        THEN RAISE(ABORT, 'Binaire trop volumineux pour la memoire flash de la carte')
    END;
END;

-- ---------------------------------------------------------------------
-- DECLENCHEUR 2 : audit automatique des compilations enregistrees
-- ---------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS trg_audit_compilation
AFTER INSERT ON compilation
FOR EACH ROW
BEGIN
    INSERT INTO journal (entite, id_entite, operation, detail)
    VALUES ('compilation', NEW.id_compilation, 'INSERT',
            'projet=' || NEW.projet || ' statut=' || NEW.statut);
END;

-- ---------------------------------------------------------------------
-- DECLENCHEUR 3 : audit des suppressions
-- ---------------------------------------------------------------------
CREATE TRIGGER IF NOT EXISTS trg_audit_suppression
AFTER DELETE ON compilation
FOR EACH ROW
BEGIN
    INSERT INTO journal (entite, id_entite, operation, detail)
    VALUES ('compilation', OLD.id_compilation, 'DELETE', 'projet=' || OLD.projet);
END;

-- ---------------------------------------------------------------------
-- VUE : donnees agregees par carte (production de donnees indisponibles)
-- ---------------------------------------------------------------------
CREATE VIEW IF NOT EXISTS v_statistiques_carte AS
SELECT  c.nom                                   AS carte,
        COUNT(cp.id_compilation)                AS nb_compilations,
        SUM(cp.statut = 'succes')               AS nb_succes,
        SUM(cp.statut = 'echec')                AS nb_echecs,
        ROUND(AVG(cp.taille_octets), 0)         AS taille_moyenne,
        MAX(cp.taille_octets)                   AS taille_max,
        ROUND(100.0 * MAX(cp.taille_octets) / (c.flash_ko * 1024), 1) AS occupation_flash_pct
FROM carte c
LEFT JOIN compilation cp ON cp.id_carte = c.id_carte
GROUP BY c.id_carte;
