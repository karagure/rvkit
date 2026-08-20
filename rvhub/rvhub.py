#!/usr/bin/env python3
"""
rvhub - Collecteur de compilations pour les projets rvkit.

Application de bureau/web legere : interface HTML accessible (RGAA 4.1)
adossee a une base de donnees SQLite dont l'integrite est garantie par
des contraintes et des declencheurs.

Architecture en trois couches independantes :
    - couche donnees        : classe Depot (pattern Repository)
    - couche metier         : classe Service
    - couche presentation   : classe Interface + serveur HTTP

Aucune dependance externe : bibliotheque standard Python uniquement.
Lancement :  python3 rvhub.py    puis ouvrir http://127.0.0.1:8080
"""

import http.server
import html
import json
import os
import sqlite3
import urllib.parse

BASE = os.path.dirname(os.path.abspath(__file__))
CHEMIN_BDD = os.path.join(BASE, "rvhub.db")
CHEMIN_SCHEMA = os.path.join(BASE, "schema.sql")


# =====================================================================
# COUCHE D'ACCES AUX DONNEES (pattern Repository)
# Le reste de l'application ignore totalement qu'il s'agit de SQLite :
# remplacer SQLite par PostgreSQL n'impacterait que cette classe.
# =====================================================================
class Depot:
    def __init__(self, chemin=CHEMIN_BDD):
        self.chemin = chemin

    def _connexion(self):
        cnx = sqlite3.connect(self.chemin)
        cnx.row_factory = sqlite3.Row
        # Les cles etrangeres ne sont pas actives par defaut sous SQLite
        cnx.execute("PRAGMA foreign_keys = ON")
        return cnx

    def initialiser(self):
        with open(CHEMIN_SCHEMA, encoding="utf-8") as f:
            script = f.read()
        with self._connexion() as cnx:
            cnx.executescript(script)

    def lister_cartes(self):
        with self._connexion() as cnx:
            return [dict(r) for r in cnx.execute(
                "SELECT * FROM carte ORDER BY nom")]

    def ajouter_carte(self, nom, arch, outil, flash_ko, ram_ko):
        with self._connexion() as cnx:
            cur = cnx.execute(
                "INSERT INTO carte (nom, cpu_arch, flash_tool, flash_ko, ram_ko)"
                " VALUES (?, ?, ?, ?, ?)",
                (nom, arch, outil, flash_ko, ram_ko))
            return cur.lastrowid

    def ajouter_compilation(self, id_carte, projet, taille, statut):
        with self._connexion() as cnx:
            cur = cnx.execute(
                "INSERT INTO compilation (id_carte, projet, taille_octets, statut)"
                " VALUES (?, ?, ?, ?)",
                (id_carte, projet, taille, statut))
            return cur.lastrowid

    def lister_compilations(self, limite=20):
        with self._connexion() as cnx:
            return [dict(r) for r in cnx.execute(
                "SELECT cp.*, c.nom AS carte, c.flash_ko"
                " FROM compilation cp JOIN carte c ON c.id_carte = cp.id_carte"
                " ORDER BY cp.compile_le DESC, cp.id_compilation DESC LIMIT ?",
                (limite,))]

    def statistiques(self):
        with self._connexion() as cnx:
            return [dict(r) for r in cnx.execute(
                "SELECT * FROM v_statistiques_carte")]

    def journal(self, limite=10):
        with self._connexion() as cnx:
            return [dict(r) for r in cnx.execute(
                "SELECT * FROM journal ORDER BY id_journal DESC LIMIT ?",
                (limite,))]


# =====================================================================
# COUCHE METIER
# Les erreurs de la base sont interceptees et traduites en messages
# exploitables : une anomalie d'acces n'interrompt jamais le service.
# =====================================================================
class Service:
    def __init__(self, depot):
        self.depot = depot

    def enregistrer_compilation(self, id_carte, projet, taille, statut):
        """Retourne (succes: bool, message: str, gravite: str)."""
        try:
            id_carte = int(id_carte)
            taille = int(taille)
        except (TypeError, ValueError):
            return False, "Carte ou taille invalide : valeur numerique attendue.", "avertissement"

        if not projet.strip():
            return False, "Le nom du projet est obligatoire.", "avertissement"

        try:
            self.depot.ajouter_compilation(id_carte, projet.strip(), taille, statut)
            return True, "Compilation enregistree.", "succes"
        except sqlite3.IntegrityError as err:
            # SQLSTATE 23000 equivalent : violation de contrainte d'integrite.
            # Gravite : avertissement. La donnee est rejetee, le service continue.
            return False, "Enregistrement refuse par la base : %s" % err, "avertissement"
        except sqlite3.DatabaseError as err:
            # Gravite : critique. Signale mais sans interruption du service.
            return False, "Erreur de base de donnees : %s" % err, "critique"

    def tableau_de_bord(self):
        return {
            "cartes": self.depot.lister_cartes(),
            "compilations": self.depot.lister_compilations(),
            "statistiques": self.depot.statistiques(),
            "journal": self.depot.journal(),
        }


# =====================================================================
# COUCHE PRESENTATION - interface conforme RGAA 4.1 / WCAG 2.1 AA
# =====================================================================
CSS = """
:root { --fond:#ffffff; --texte:#1a1a1a; --bord:#d0d0d0; --accent:#0b5fa5;
        --succes:#0a6b2e; --alerte:#a4262c; }
* { box-sizing:border-box; }
body { margin:0; font-family:system-ui,Segoe UI,Arial,sans-serif;
       background:var(--fond); color:var(--texte); line-height:1.5; }
.lien-evitement { position:absolute; left:-9999px; }
.lien-evitement:focus { position:static; display:block; padding:.5rem;
       background:var(--accent); color:#fff; }
header { background:var(--texte); color:#fff; padding:1rem; }
header h1 { margin:0; font-size:1.3rem; }
main { max-width:60rem; margin:0 auto; padding:1rem; }
h2 { font-size:1.1rem; border-bottom:2px solid var(--bord); padding-bottom:.3rem;
     margin-top:2rem; }
table { border-collapse:collapse; width:100%; margin:.5rem 0 1rem; }
caption { text-align:left; font-weight:bold; padding:.4rem 0; }
th,td { border:1px solid var(--bord); padding:.45rem .6rem; text-align:left;
        font-size:.92rem; }
th { background:#f0f2f4; }
form { border:1px solid var(--bord); padding:1rem; border-radius:6px; }
.champ { margin-bottom:.8rem; }
label { display:block; font-weight:600; margin-bottom:.2rem; }
input,select { width:100%; padding:.45rem; border:1px solid #767676;
        border-radius:4px; font-size:1rem; }
input:focus,select:focus,button:focus,a:focus { outline:3px solid #b35c00;
        outline-offset:2px; }
button { background:var(--accent); color:#fff; border:0; padding:.6rem 1.2rem;
        font-size:1rem; border-radius:4px; cursor:pointer; }
.message { padding:.7rem 1rem; border-radius:4px; margin:1rem 0;
        border-left:6px solid; }
.succes { background:#e8f5ec; border-color:var(--succes); color:var(--succes); }
.avertissement,.critique { background:#fdeaea; border-color:var(--alerte);
        color:var(--alerte); }
.aide { font-size:.85rem; color:#5a5a5a; }
"""


class Interface:
    """Rend les pages HTML. Conformite RGAA :
    - structure semantique (header/main/section, titres hierarchises)
    - lien d'evitement en premier element focusable
    - contrastes superieurs au ratio 4,5:1 exige
    - chaque champ possede un label explicitement associe (for/id)
    - l'information n'est jamais portee par la seule couleur (prefixes texte)
    - tableaux dotes d'une legende et d'en-tetes declares
    - focus visible renforce, navigation clavier complete
    """

    @staticmethod
    def page(contenu, message=None):
        bloc = ""
        if message:
            texte, gravite = message
            prefixe = "Succes : " if gravite == "succes" else "Erreur : "
            bloc = ('<p class="message %s" role="status">%s%s</p>'
                    % (gravite, prefixe, html.escape(texte)))
        return """<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>rvhub - collecteur de compilations rvkit</title>
<style>%s</style>
</head>
<body>
<a class="lien-evitement" href="#contenu">Aller au contenu principal</a>
<header>
  <h1>rvhub &mdash; collecteur de compilations rvkit</h1>
</header>
<main id="contenu">
%s
%s
</main>
</body>
</html>""" % (CSS, bloc, contenu)

    @staticmethod
    def tableau_de_bord(donnees):
        cartes = donnees["cartes"]

        options = "".join(
            '<option value="%d">%s</option>' % (c["id_carte"], html.escape(c["nom"]))
            for c in cartes)

        lignes_stats = "".join(
            "<tr><th scope=\"row\">%s</th><td>%s</td><td>%s</td><td>%s</td>"
            "<td>%s</td><td>%s %%</td></tr>" % (
                html.escape(s["carte"]), s["nb_compilations"] or 0,
                s["nb_succes"] or 0, s["nb_echecs"] or 0,
                s["taille_moyenne"] if s["taille_moyenne"] is not None else "-",
                s["occupation_flash_pct"] if s["occupation_flash_pct"] is not None else "-")
            for s in donnees["statistiques"])

        lignes_comp = "".join(
            "<tr><td>%s</td><td>%s</td><td>%s o</td><td>%s</td><td>%s</td></tr>" % (
                html.escape(c["projet"]), html.escape(c["carte"]),
                c["taille_octets"],
                "Succes" if c["statut"] == "succes" else "Echec",
                html.escape(str(c["compile_le"])))
            for c in donnees["compilations"]) or \
            '<tr><td colspan="5">Aucune compilation enregistree.</td></tr>'

        lignes_journal = "".join(
            "<tr><td>%s</td><td>%s</td><td>%s</td><td>%s</td></tr>" % (
                html.escape(j["entite"]), html.escape(j["operation"]),
                html.escape(j["detail"] or ""), html.escape(str(j["effectue_le"])))
            for j in donnees["journal"]) or \
            '<tr><td colspan="4">Journal vide.</td></tr>'

        lignes_cartes = "".join(
            "<tr><th scope=\"row\">%s</th><td>%s</td><td>%s</td><td>%s Ko</td>"
            "<td>%s Ko</td></tr>" % (
                html.escape(c["nom"]), html.escape(c["cpu_arch"]),
                html.escape(c["flash_tool"]), c["flash_ko"], c["ram_ko"])
            for c in cartes)

        return """
<section aria-labelledby="t-saisie">
<h2 id="t-saisie">Enregistrer une compilation</h2>
<form method="post" action="/compilation">
  <div class="champ">
    <label for="carte">Carte cible</label>
    <select id="carte" name="id_carte" required>%s</select>
  </div>
  <div class="champ">
    <label for="projet">Nom du projet</label>
    <input type="text" id="projet" name="projet" required
           aria-describedby="aide-projet">
    <p class="aide" id="aide-projet">Nom du dossier genere par la commande de creation.</p>
  </div>
  <div class="champ">
    <label for="taille">Taille du binaire produit, en octets</label>
    <input type="number" id="taille" name="taille" min="0" required
           aria-describedby="aide-taille">
    <p class="aide" id="aide-taille">Un binaire depassant la memoire flash de la carte
       sera refuse par la base de donnees.</p>
  </div>
  <div class="champ">
    <label for="statut">Resultat de la compilation</label>
    <select id="statut" name="statut">
      <option value="succes">Succes</option>
      <option value="echec">Echec</option>
    </select>
  </div>
  <button type="submit">Enregistrer</button>
</form>
</section>

<section aria-labelledby="t-stats">
<h2 id="t-stats">Donnees agregees par carte</h2>
<table>
  <caption>Statistiques calculees a partir des compilations enregistrees</caption>
  <thead><tr><th scope="col">Carte</th><th scope="col">Compilations</th>
  <th scope="col">Succes</th><th scope="col">Echecs</th>
  <th scope="col">Taille moyenne</th><th scope="col">Occupation flash</th></tr></thead>
  <tbody>%s</tbody>
</table>
</section>

<section aria-labelledby="t-comp">
<h2 id="t-comp">Dernieres compilations</h2>
<table>
  <caption>Historique des compilations, de la plus recente a la plus ancienne</caption>
  <thead><tr><th scope="col">Projet</th><th scope="col">Carte</th>
  <th scope="col">Taille</th><th scope="col">Resultat</th>
  <th scope="col">Date</th></tr></thead>
  <tbody>%s</tbody>
</table>
</section>

<section aria-labelledby="t-cartes">
<h2 id="t-cartes">Cartes referencees</h2>
<table>
  <caption>Referentiel des cibles materielles supportees</caption>
  <thead><tr><th scope="col">Carte</th><th scope="col">Architecture</th>
  <th scope="col">Outil de flash</th><th scope="col">Flash</th>
  <th scope="col">RAM</th></tr></thead>
  <tbody>%s</tbody>
</table>
</section>

<section aria-labelledby="t-journal">
<h2 id="t-journal">Journal d'audit</h2>
<table>
  <caption>Operations tracees automatiquement par les declencheurs de la base</caption>
  <thead><tr><th scope="col">Entite</th><th scope="col">Operation</th>
  <th scope="col">Detail</th><th scope="col">Date</th></tr></thead>
  <tbody>%s</tbody>
</table>
</section>
""" % (options, lignes_stats, lignes_comp, lignes_cartes, lignes_journal)


# =====================================================================
# SERVEUR HTTP
# =====================================================================
depot = Depot()
service = Service(depot)


class Gestionnaire(http.server.BaseHTTPRequestHandler):
    message = None

    def _envoyer(self, corps, code=200, mime="text/html; charset=utf-8"):
        donnees = corps.encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", mime)
        self.send_header("Content-Length", str(len(donnees)))
        self.end_headers()
        self.wfile.write(donnees)

    def do_GET(self):
        if self.path.startswith("/api/statistiques"):
            # Export des donnees agregees au format JSON (flux sortant)
            self._envoyer(json.dumps(depot.statistiques(), ensure_ascii=False,
                                     indent=2), mime="application/json; charset=utf-8")
            return
        contenu = Interface.tableau_de_bord(service.tableau_de_bord())
        page = Interface.page(contenu, Gestionnaire.message)
        Gestionnaire.message = None
        self._envoyer(page)

    def do_POST(self):
        taille = int(self.headers.get("Content-Length", 0))
        donnees = urllib.parse.parse_qs(self.rfile.read(taille).decode("utf-8"))
        ok, msg, gravite = service.enregistrer_compilation(
            donnees.get("id_carte", [""])[0],
            donnees.get("projet", [""])[0],
            donnees.get("taille", [""])[0],
            donnees.get("statut", ["succes"])[0])
        Gestionnaire.message = (msg, gravite)
        self.send_response(303)
        self.send_header("Location", "/")
        self.end_headers()

    def log_message(self, *args):
        pass  # journalisation desactivee pour garder la console lisible


def amorcer():
    """Cree la base et insere le referentiel des cartes si absent."""
    depot.initialiser()
    if not depot.lister_cartes():
        depot.ajouter_carte("ch32v003", "riscv32", "wlink", 16, 2)
        depot.ajouter_carte("esp32-c3", "riscv32", "esptool", 4096, 400)
        print("Base initialisee avec le referentiel des cartes.")


if __name__ == "__main__":
    amorcer()
    port = 8080
    print("rvhub demarre. Ouvrir http://127.0.0.1:%d" % port)
    http.server.HTTPServer(("127.0.0.1", port), Gestionnaire).serve_forever()
