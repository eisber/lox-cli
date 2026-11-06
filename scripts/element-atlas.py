#!/usr/bin/env python3
"""Generate a private, version-pinned element atlas without executing Config."""

import argparse
from collections import Counter
import csv
import html
import importlib.util
import io
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "atlas_ux_contracts", Path(__file__).with_name("verify-ux-contracts.py"))
GATES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATES)
SHA256 = re.compile(r"[0-9a-f]{64}")


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"),
                      object_pairs_hook=GATES._object,
                      parse_constant=GATES._nonfinite)


def source_ref(path, root, role):
    path = Path(path).resolve()
    if not path.is_relative_to(root):
        raise ValueError(f"{role}: source escapes root")
    return {"path": str(path.relative_to(root)).replace("/", "\\"),
            "sha256": GATES.digest(path), "role": role}


def fraction(count, total):
    return {"count": count, "total": total, "remaining": total - count,
            "percentage": round(100 * count / total, 2) if total else None}


def validate_application(application):
    if (type(application) is not dict
            or type(application.get("version")) is not str
            or not application["version"]
            or type(application.get("sha256")) is not str
            or not SHA256.fullmatch(application["sha256"])):
        raise ValueError("application needs version and executable SHA256")


def validate_inventory(inventory):
    if (type(inventory) is not dict or type(inventory.get("source")) is not dict
            or type(inventory.get("blocks")) is not list):
        raise ValueError("inventory needs source and blocks")
    sha256 = inventory["source"].get("executable_sha256")
    if type(sha256) is not str or not SHA256.fullmatch(sha256):
        raise ValueError("inventory executable SHA256 is invalid")
    indices = set()
    for row in inventory["blocks"]:
        if type(row) is not dict:
            raise ValueError("catalog record must be an object")
        index = row.get("catalog_index")
        if type(index) is not int or index < 0 or index in indices:
            raise ValueError("catalog_index must be a unique nonnegative integer")
        indices.add(index)
        for key in ("xml_type", "control_type"):
            if row.get(key) is not None and type(row[key]) is not str:
                raise ValueError(f"catalog {key} must be a string or null")
        parameters = row.get("parameters", [])
        if type(parameters) is not list or any(type(p) is not dict for p in parameters):
            raise ValueError("catalog parameters must be objects")
        for parameter in parameters:
            keys = parameter.get("known_xml_keys", [])
            if type(keys) is not list or any(type(key) is not str for key in keys):
                raise ValueError("parameter known_xml_keys must be strings")
    return indices


def read_sidecar(path, root, application, inventory_ref, kind, indices):
    document = load_json(path)
    if type(document) is not dict or document.get("schema_version") != 1:
        raise ValueError(f"{kind}: unsupported sidecar")
    if document.get("application") != application:
        raise ValueError(f"{kind}: executable/version mismatch")
    sources = document.get("sources")
    if type(sources) is not list or not sources:
        raise ValueError(f"{kind}: missing source pins")
    inventory_pinned = False
    for source in sources:
        if (type(source) is not dict or type(source.get("path")) is not str
                or type(source.get("sha256")) is not str
                or not SHA256.fullmatch(source["sha256"])):
            raise ValueError(f"{kind}: invalid source reference")
        target = GATES._artifact_path(root, source["path"])
        if GATES.digest(target) != source["sha256"]:
            raise ValueError(f"{kind}: source SHA256 mismatch: {source['path']}")
        if (source.get("role") == "inventory"
                and target == GATES._artifact_path(root, inventory_ref["path"])
                and source["sha256"] == inventory_ref["sha256"]):
            inventory_pinned = True
    if not inventory_pinned:
        raise ValueError(f"{kind}: sidecar does not pin this inventory")
    if kind == "native":
        classes, joins = document.get("classes"), document.get("joins")
        if type(classes) is not list or type(joins) is not list:
            raise ValueError("native: classes and joins must be arrays")
        ids = [row.get("class_id") for row in classes if type(row) is dict]
        source_indices = [row.get("source_index") for row in classes if type(row) is dict]
        if (len(ids) != len(classes) or any(type(key) is not str or not key for key in ids)
                or len(set(ids)) != len(ids)
                or any(type(i) is not int or i < 0 for i in source_indices)
                or len(set(source_indices)) != len(source_indices)):
            raise ValueError("native: classes need unique identities and source indices")
        for join in joins:
            if (type(join) is not dict or type(join.get("catalog_index")) is not int
                    or join["catalog_index"] not in indices or join.get("class_id") not in ids
                    or join.get("status") not in ("exact", "alias", "candidate", "ambiguous")):
                raise ValueError("native: invalid catalog/class join")
    else:
        mechanisms = document.get("mechanisms")
        if type(mechanisms) is not list:
            raise ValueError("UX: mechanisms must be an array")
        ids = []
        for mechanism in mechanisms:
            if (type(mechanism) is not dict
                    or type(mechanism.get("mechanism_id")) is not str
                    or not mechanism["mechanism_id"]
                    or type(mechanism.get("catalog_indices")) is not list
                    or any(type(i) is not int or i not in indices
                           for i in mechanism["catalog_indices"])):
                raise ValueError("UX: invalid mechanism identity/membership")
            ids.append(mechanism["mechanism_id"])
        if len(set(ids)) != len(ids):
            raise ValueError("UX: duplicate mechanism identity")
    return document


def read_reports(contracts, results, root, application):
    by_digest = {}
    for path in results:
        result = load_json(path)
        GATES.validate_document(result, "ux-results.schema.json")
        key = result["contract_sha256"]
        if key in by_digest:
            raise ValueError("duplicate results for one contract snapshot")
        by_digest[key] = path
    reports, snapshots = [], set()
    for path in contracts:
        snapshot = GATES.digest(path)
        if snapshot in snapshots:
            raise ValueError("duplicate contract snapshot")
        snapshots.add(snapshot)
        report = GATES.verify_files(path, by_digest.pop(snapshot, None),
                                   root, application)
        report["contract"] = source_ref(path, root, "contract")
        if report["results"]:
            report["results"] = source_ref(report["results"], root, "results")
        reports.append(report)
    if by_digest:
        raise ValueError("results do not match a supplied contract snapshot")
    return reports


def read_reference(path, root, application):
    document = load_json(path)
    if type(document) is not dict:
        raise ValueError("reference must be a JSON object")
    identity = document.get("application")
    if identity is None and type(document.get("provenance")) is dict:
        provenance = document["provenance"]
        identity = {"version": provenance.get("version"),
                    "sha256": provenance.get("target_sha256")}
    if identity != application:
        raise ValueError("reference executable/version mismatch")
    return {"source": source_ref(path, root, "reference"),
            "block_type": document.get("block_type", document.get("xml_type")),
            "control_type": document.get("control_type"),
            "status": "reference_snapshot_not_verified_execution",
            "document": document}


def identity_matches(row, reference):
    xml_type = reference["block_type"]
    control_type = reference["control_type"]
    return (xml_type is not None and row.get("xml_type") == xml_type
            and (control_type is None or row.get("control_type") == str(control_type)))


def build_atlas(inventory, inventory_ref, application, reports=(), native=None,
                ux=None, references=(), connector_map=None, connector_ref=None):
    indices = validate_inventory(inventory)
    validate_application(application)
    if inventory["source"]["executable_sha256"] != application["sha256"]:
        raise ValueError("inventory executable mismatch")
    compatible = [report for report in reports if report["application"] == application]
    evidence = {index: [] for index in indices}
    unmatched_contracts = []
    for report in compatible:
        enriched = GATES.enrich_inventory(inventory, [report])
        unmatched_contracts.extend(enriched["evidence_summary"]["diagnostics"])
        for row in enriched["blocks"]:
            if row["verified_evidence"]["report"] is not None:
                evidence[row["catalog_index"]].append(report)
    classes = native["classes"] if native else []
    joins = native["joins"] if native else []
    mechanisms = ux["mechanisms"] if ux else []
    by_index = {index: [] for index in indices}
    by_mechanism = {index: [] for index in indices}
    source_records = {row["catalog_index"]: row for row in inventory["blocks"]}
    for join in joins:
        for field in ("xml_type", "control_type"):
            if field in join and join[field] != source_records[join["catalog_index"]].get(field):
                raise ValueError(f"native join {field} disagrees with catalog record")
        by_index[join["catalog_index"]].append(join)
    for mechanism in mechanisms:
        for index in set(mechanism["catalog_indices"]):
            by_mechanism[index].append(mechanism["mechanism_id"])
    records, used_references = [], set()
    for row in sorted(inventory["blocks"], key=lambda item: item["catalog_index"]):
        index = row["catalog_index"]
        linked_references = []
        for number, reference in enumerate(references):
            if (identity_matches(row, reference)
                    and sum(identity_matches(other, reference)
                            for other in inventory["blocks"]) == 1):
                linked_references.append(number)
                used_references.add(number)
        parameters = row.get("parameters", [])
        mapped = sum(bool(parameter.get("known_xml_keys")) for parameter in parameters)
        connector_data = (connector_map or {}).get(row.get("xml_type"))
        connector_count = None
        if connector_data is not None:
            if (type(connector_data) is not dict
                    or type(connector_data.get("c")) is not list):
                raise ValueError("connector metadata needs a connector array")
            connector_count = len(connector_data["c"])
        overlays = []
        for number in linked_references:
            document = references[number]["document"]
            expected = document.get("expected_native_ordered_connector_count")
            if expected is not None:
                overlays.append({
                    "reference": number, "current_cli_connector_count": connector_count,
                    "declared_native_count": expected,
                    "count_matches": connector_count == expected,
                    "scope": "metadata count only; no GUI/runtime promotion",
                })
        snapshots = evidence[index]
        evidence_status = "unverified"
        if len(snapshots) == 1:
            evidence_status = "complete" if snapshots[0]["gate"]["complete"] else "partial"
        elif snapshots:
            evidence_status = "independent_snapshots"
        statuses = {join["status"] for join in by_index[index]}
        native_status = ("not_assessed" if native is None else next(
            (status for status in ("exact", "alias", "ambiguous", "candidate")
             if status in statuses), "unresolved"))
        records.append({
            "record_id": f"catalog:{inventory_ref['sha256']}:{index}",
            "catalog_index": index, "control_type": row.get("control_type"),
            "xml_type": row.get("xml_type"), "name": row.get("name"),
            "identity_status": row.get("identity_status", "unknown"),
            "kind": "not_classified",
            "families": row.get("families", []),
            "advanced_candidate": row.get("advanced_candidate"),
            "parameter_mapping": fraction(mapped, len(parameters)),
            "parameters": parameters,
            "entry_counts": row.get("entry_counts", {}),
            "storage_observations": row.get("storage_observations", {}),
            "historical_cli_snapshot": row.get("cli"),
            "historical_ux_snapshot": row.get("ux_parity"),
            "historical_native_anchors": row.get("native_anchors", []),
            "native_joins": by_index[index],
            "native_association_status": native_status,
            "ux_mechanism_ids": sorted(by_mechanism[index]),
            "current_cli_metadata": {"source": connector_ref,
                                     "connector_count": connector_count},
            "contract_evidence": {
                "status": evidence_status, "snapshots": snapshots,
                "report": snapshots[0] if len(snapshots) == 1 else None},
            "reference_indices": linked_references, "metadata_overlays": overlays,
            "overall_percentage": None,
        })
    counts = Counter(row.get("control_type") for row in records
                     if row.get("control_type") is not None)
    class_ids_with_join = {join["class_id"] for join in joins}
    proven_class_ids = {join["class_id"] for join in joins if join["status"] in ("exact", "alias")}
    proven_catalog_indices = {join["catalog_index"] for join in joins
                              if join["status"] in ("exact", "alias")}
    parameter_total = sum(record["parameter_mapping"]["total"] for record in records)
    parameter_mapped = sum(record["parameter_mapping"]["count"] for record in records)
    native_records = sum(bool(record["native_joins"]) for record in records)
    join_statuses = Counter()
    for record in records:
        join_statuses[record["native_association_status"]] += 1
    diagnostic_reports = [report for report in reports if not report["gate"]["input_valid"]]
    valid_reports = [report for report in compatible if report["gate"]["input_valid"]]
    valid_capabilities = sum(len(report["capabilities"]) for report in valid_reports)
    summary = {
        "catalog_records": len(records), "distinct_control_type_ids": len(counts),
        "xml_name_annotation": fraction(sum(record["xml_type"] is not None
                                           for record in records), len(records)),
        "parameter_xml_key_mapping": fraction(parameter_mapped, parameter_total),
        "native_class_records": len(classes) if native else None,
        "catalog_records_with_native_leads": fraction(native_records, len(records))
        if native else None,
        "catalog_native_association_status": dict(sorted(join_statuses.items()))
        if native else None,
        "catalog_records_with_proven_static_identity": fraction(
            len(proven_catalog_indices), len(records)) if native else None,
        "native_classes_with_proven_catalog_identity": fraction(
            len(proven_class_ids), len(classes)) if native else None,
        "ux_mechanisms": len(mechanisms) if ux else None,
        "ux_extraction_status": ux.get("summary", {}).get("status") if ux else None,
        "ux_evidence_status": ux.get("summary", {}).get("evidence_status") if ux else None,
        "ux_unresolved_evidence_diagnostics": ux.get("summary", {}).get(
            "incomplete_evidence_diagnostics") if ux else None,
        "catalog_records_with_ux_mechanism_leads": fraction(
            sum(bool(record["ux_mechanism_ids"]) for record in records), len(records))
        if ux else None,
        "declared_contracts": len(reports),
        "declared_contract_capabilities": sum(len(report["capabilities"]) for report in reports),
        "input_valid_contract_capabilities": valid_capabilities,
        "contract_stages": {
            stage: fraction(sum(report["stages"][stage]["verified"] for report in valid_reports),
                            valid_capabilities)
            for stage in GATES.STAGES},
        "contract_stage_denominator": (
            "declared capability instances in input-valid independent snapshots; "
            "overlapping capabilities are not deduplicated or combined"),
        "invalid_contract_reports": len(diagnostic_reports),
        "overall_percentage": None,
        "denominator_status": "source inventories only; full UX/behavior scope is unknown",
    }
    return {
        "schema_version": 1, "application": application,
        "generator": {"name": Path(__file__).name, "sha256": GATES.digest(__file__),
                      "evidence_verifier_sha256": GATES.digest(SPEC.origin)},
        "scope": {"status": "incomplete", "overall_percentage": None,
                  "native_execution_rollup": "not_inferred_from_static_or_UX_evidence",
                  "catalog_kind_classification": "not_complete"},
        "sources": [inventory_ref] + ([connector_ref] if connector_ref else []),
        "summary": summary, "records": records, "native_classes": classes,
        "native_map": {key: value for key, value in native.items()
                       if key not in ("classes", "joins")} if native else None,
        "ux_map": ux, "contract_reports": list(reports),
        "references": list(references),
        "exceptions": {
            "duplicate_control_type_ids": [
                {"control_type": key, "catalog_indices": [
                    record["catalog_index"] for record in records
                    if record["control_type"] == key]}
                for key in sorted(counts) if counts[key] > 1],
            "without_xml_name": [record["catalog_index"] for record in records
                                 if record["xml_type"] is None],
            "without_native_lead": [record["catalog_index"] for record in records
                                    if not record["native_joins"]] if native else None,
            "candidate_only_native_joins": [
                record["catalog_index"] for record in records
                if record["native_joins"] and all(
                    join["status"] in ("candidate", "ambiguous")
                    for join in record["native_joins"])],
            "native_classes_without_catalog_lead": [
                row["class_id"] for row in classes if row["class_id"] not in class_ids_with_join]
            if native else None,
            "native_classes_without_proven_catalog_identity": [
                row["class_id"] for row in classes if row["class_id"] not in proven_class_ids]
            if native else None,
            "invalid_contracts": [report["contract"] for report in diagnostic_reports],
            "unmatched_contracts": unmatched_contracts + [
                {"code": "contract_application_mismatch", "contract": report["contract"]}
                for report in reports if report["application"] != application],
            "unmatched_references": sorted(set(range(len(references))) - used_references),
        },
        "limitations": [
            "Catalog records and native classes are different, incomplete populations.",
            "A numeric ControlType ID is not a unique element identity.",
            "Class-name candidates and shared metadata do not prove factory or UX bindings.",
            "Original inventory observations remain historical, not current CLI support claims.",
            "References and newer metadata overlays never manufacture executed passes.",
            "An absent verified case is not evidence that a control has never been exercised.",
        ],
    }


def native_name(row):
    return row.get("native_class") or row.get("type", {}).get("display_name") or row["class_id"]


def primary_values(rows, key):
    return ", ".join(sorted({str(table[key]) for row in rows
                             for table in row.get("primary_tables", []) if key in table}))


def catalog_cells(record, classes):
    native = [classes[join["class_id"]] for join in record["native_joins"]]
    evidence = record["contract_evidence"]["status"]
    for report in record["contract_evidence"]["snapshots"]:
        evidence += " | " + report["contract"]["path"] + ": "
        evidence += ("invalid" if not report["gate"]["input_valid"] else "; ".join(
            f"{stage}={report['stages'][stage]['verified']}/"
            f"{report['stages'][stage]['declared']}"
            for stage in ("binding_known", "unit_test", "gui_oracle")))
    return [
        record["catalog_index"], record["control_type"], record["xml_type"], record["name"],
        ", ".join(record["families"]),
        f"{record['parameter_mapping']['count']}/{record['parameter_mapping']['total']}",
        record["native_association_status"],
        ", ".join(native_name(row) for row in native),
        primary_values(native, "vtable"), primary_values(native, "target"),
        ", ".join(record["ux_mechanism_ids"]),
        record["current_cli_metadata"]["connector_count"], evidence,
    ]


CATALOG_HEADERS = ["CATALOG INDEX", "CONTROL TYPE", "XML TYPE", "NAME", "FAMILIES",
                   "PARAMETER KEYS", "NATIVE JOIN", "NATIVE CLASS", "PRIMARY VTABLE",
                   "PRIMARY +0x50 CANDIDATE", "UX MECHANISMS",
                   "CLI CONNECTORS", "LINKED UX CONTRACT"]
NATIVE_HEADERS = ["SOURCE INDEX", "NATIVE CLASS", "PRIMARY VTABLE", "PRIMARY +0x50 CANDIDATE",
                  "CATALOG ASSOCIATION", "CLASS ID"]


def native_cells(row, joins):
    states = {join["status"] for join in joins if join["class_id"] == row["class_id"]}
    status = ("unjoined" if not states else "proven_static_identity"
              if states & {"exact", "alias"} else "candidate_only")
    return [row["source_index"], native_name(row), primary_values([row], "vtable"),
            primary_values([row], "target"), status, row["class_id"]]


def csv_text(headers, rows):
    stream = io.StringIO(newline="")
    writer = csv.writer(stream, lineterminator="\n")
    writer.writerow(headers)
    writer.writerows(rows)
    return stream.getvalue()


def render_html(atlas):
    def escape(value):
        return html.escape("" if value is None else str(value), quote=True)

    def table(headers, rows):
        return ("<table><thead><tr>"
                + "".join(f"<th>{escape(header)}</th>" for header in headers)
                + "</tr></thead><tbody>"
                + "".join("<tr>" + "".join(f"<td>{escape(cell)}</td>" for cell in row)
                          + "</tr>" for row in rows) + "</tbody></table>")

    classes = {row["class_id"]: row for row in atlas["native_classes"]}
    joins = [join for record in atlas["records"] for join in record["native_joins"]]
    catalog = [catalog_cells(record, classes) for record in atlas["records"]]
    native_rows = [native_cells(row, joins) for row in atlas["native_classes"]]
    mechanisms = (atlas["ux_map"] or {}).get("mechanisms", [])
    mechanism_rows = [[row["mechanism_id"], row.get("kind"), row.get("status"),
                       ", ".join(map(str, row["catalog_indices"])),
                       json.dumps(row.get("native_addresses", []))]
                      for row in mechanisms]
    summary = escape(json.dumps(atlas["summary"], indent=2))
    return """<!doctype html>
<html lang="en"><meta charset="utf-8"><title>Loxone element atlas</title>
<style>
body{font:14px system-ui;margin:2em;color:#20242a}table{border-collapse:collapse;width:100%}
td,th{border:1px solid #ddd;padding:.4em;text-align:left;overflow-wrap:anywhere}
th{background:#eee;position:sticky;top:0}input{padding:.6em;width:30em;max-width:90%}
pre{white-space:pre-wrap}section{margin-bottom:3em}tr[hidden]{display:none}
</style>
<h1>Version-pinned element atlas</h1>
<p>Static candidates, historical observations, and verified cases are different evidence.
Overall UX/behavior completion is unknown. All source records remain in atlas.json.</p>
<p>""" + escape(atlas["application"]["version"]) + " / " + escape(
        atlas["application"]["sha256"]) + """</p><pre>""" + summary + """</pre>
<label>Filter tables <input id="filter" type="search" placeholder="XML type, class, ID, family..."></label>
<section><h2>Catalog records</h2>""" + table(CATALOG_HEADERS, catalog) + """</section>
<section><h2>Native classes (separate population)</h2>""" + table(
        NATIVE_HEADERS, native_rows) + """</section>
<section><h2>Shared UX mechanism leads</h2>""" + table(
        ["MECHANISM", "KIND", "STATUS", "CATALOG INDICES", "NATIVE ADDRESS LEADS"],
        mechanism_rows) + """</section>
<h2>Exceptions and source limitations</h2><pre>""" + escape(json.dumps(
        {"exceptions": atlas["exceptions"], "limitations": atlas["limitations"]},
        indent=2)) + """</pre>
<script>
document.getElementById('filter').addEventListener('input',function(){
const query=this.value.toLocaleLowerCase();
document.querySelectorAll('tbody tr').forEach(row=>{
row.hidden=!row.textContent.toLocaleLowerCase().includes(query);
});
});
</script></html>
"""


def write_atlas(atlas, output):
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    classes = {row["class_id"]: row for row in atlas["native_classes"]}
    joins = [join for record in atlas["records"] for join in record["native_joins"]]
    files = {
        "atlas.json": json.dumps(atlas, indent=2, ensure_ascii=True) + "\n",
        "index.html": render_html(atlas),
        "catalog.csv": csv_text(CATALOG_HEADERS, [
            catalog_cells(record, classes) for record in atlas["records"]]),
        "native-classes.csv": csv_text(NATIVE_HEADERS, [
            native_cells(row, joins) for row in atlas["native_classes"]]),
    }
    for name, content in files.items():
        (output / name).write_text(content, encoding="utf-8", newline="\n")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--inventory", type=Path, required=True)
    parser.add_argument("--application-version", required=True)
    parser.add_argument("--native-map", type=Path)
    parser.add_argument("--ux-map", type=Path)
    parser.add_argument("--contract", type=Path, action="append", default=[])
    parser.add_argument("--results", type=Path, action="append", default=[])
    parser.add_argument("--reference", type=Path, action="append", default=[])
    parser.add_argument("--connector-map", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        root = args.root.resolve()
        inventory = load_json(args.inventory)
        indices = validate_inventory(inventory)
        application = {"version": args.application_version,
                       "sha256": inventory["source"]["executable_sha256"]}
        validate_application(application)
        inventory_ref = source_ref(args.inventory, root, "inventory")
        native = read_sidecar(args.native_map, root, application, inventory_ref,
                              "native", indices) if args.native_map else None
        ux = read_sidecar(args.ux_map, root, application, inventory_ref,
                          "ux", indices) if args.ux_map else None
        reports = read_reports(args.contract, args.results, root, application)
        references = [read_reference(path, root, application) for path in args.reference]
        connectors = load_json(args.connector_map) if args.connector_map else None
        if connectors is not None and type(connectors) is not dict:
            raise ValueError("connector map must be an object")
        connector_ref = source_ref(args.connector_map, root, "current_cli_metadata") \
            if args.connector_map else None
        atlas = build_atlas(inventory, inventory_ref, application, reports, native, ux,
                            references, connectors, connector_ref)
        atlas["sources"] += [
            source_ref(path, root, role)
            for path, role in ((args.native_map, "native_map"), (args.ux_map, "ux_map"))
            if path is not None]
        write_atlas(atlas, args.output)
        print(json.dumps({"output": str(args.output), "summary": atlas["summary"],
                          "invalid_contracts": atlas["exceptions"]["invalid_contracts"]},
                         indent=2))
        return 2 if atlas["summary"]["invalid_contract_reports"] else 0
    except (OSError, ValueError, KeyError) as error:
        print(json.dumps({"error": str(error)}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
