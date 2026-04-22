#!/usr/bin/env python3
"""
Semantic Eval — LLM-based behavioral verification.

Instead of checking exact block structures, asks an LLM:
"Given this utterance and the config changes, does the automation
 work as requested?"

Usage:
  python3 tests/eval/semantic_eval.py <case-id> <result.Loxone>
  python3 tests/eval/semantic_eval.py --batch <results-dir> --output report.json
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
EVAL_DIR = SCRIPT_DIR.parent
FIXTURE = EVAL_DIR / "fixture.Loxone"
CASES_INDEX = EVAL_DIR / "cases-index.json"


def load_case(case_id):
    with open(CASES_INDEX) as f:
        index = json.load(f)
    
    for category, meta in index['categories'].items():
        case_file = EVAL_DIR / meta['file']
        with open(case_file) as f:
            cases = json.load(f)
            for c in cases:
                if c['id'] == case_id:
                    return c
    return None


def get_config_diff(fixture_path, result_path):
    """Get a human-readable diff between fixture and result."""
    lox = _find_lox()
    
    # Get stats of result
    r = subprocess.run(lox + ['config', 'stats', str(result_path)],
                       capture_output=True, text=True, timeout=30)
    stats = r.stdout
    
    # Get check results
    r = subprocess.run(lox + ['config', 'check', str(result_path)],
                       capture_output=True, text=True, timeout=30)
    check = r.stdout
    
    # Get describe (shows all controls)
    r = subprocess.run(lox + ['config', 'describe', str(result_path)],
                       capture_output=True, text=True, timeout=30)
    describe = r.stdout
    
    # Get diff
    r = subprocess.run(lox + ['config', 'diff', str(fixture_path), str(result_path)],
                       capture_output=True, text=True, timeout=30)
    diff = r.stdout
    
    return {
        'stats': stats,
        'check': check,
        'describe': describe,
        'diff': diff,
    }


def build_judge_prompt(case, config_info):
    """Build the prompt for the LLM judge."""
    return f"""You are evaluating whether a Loxone home automation config correctly implements a user's request.

## User Request
"{case['utterance']}"

## Config Changes (diff from baseline)
{config_info['diff']}

## Automation Check (block completeness)
{config_info['check']}

## Current Controls
{config_info['describe'][:3000]}

## Evaluation Criteria
Score each dimension 0-100:

1. **Completeness** (0-100): Does it implement ALL parts of the request?
   - 100 = every requirement addressed
   - 50 = some requirements implemented, some missing
   - 0 = nothing implemented

2. **Correctness** (0-100): Are the implementations functionally correct?
   - 100 = all logic correct, right thresholds, right wiring
   - 50 = partially correct, some logic errors
   - 0 = fundamentally wrong approach

3. **No Side Effects** (0-100): Does it avoid unwanted behavior?
   - 100 = only affects what was requested
   - 80 = minor extra blocks but no harm
   - 50 = affects unrelated controls
   - 0 = breaks existing functionality

4. **Validity** (0-100): Is the config technically valid?
   - 100 = all blocks wired, params set, check passes
   - 50 = some unwired blocks or missing params
   - 0 = broken config

Output ONLY a JSON object with this exact format:
{{"completeness": <0-100>, "correctness": <0-100>, "no_side_effects": <0-100>, "validity": <0-100>, "pass": <true/false>, "reasoning": "<1-2 sentence explanation>"}}

A case passes if completeness >= 60 AND correctness >= 60 AND validity >= 80.
"""


def _find_lox():
    lox = Path('/home/amy/src/lox/target/debug/lox')
    if lox.exists():
        return [str(lox)]
    return ['cargo', 'run', '--quiet', '--']


def main():
    parser = argparse.ArgumentParser(description='Semantic Eval')
    parser.add_argument('case_id', nargs='?', help='Case ID')
    parser.add_argument('result', nargs='?', help='Result .Loxone file')
    parser.add_argument('--batch', help='Evaluate all results in directory')
    parser.add_argument('--output', default='semantic-report.json', help='Output report')
    args = parser.parse_args()

    if args.batch:
        # Batch mode — evaluate all
        fixture_size = os.path.getsize(str(FIXTURE))
        results = []
        
        for f in sorted(os.listdir(args.batch)):
            if not f.endswith('.Loxone'):
                continue
            path = os.path.join(args.batch, f)
            if os.path.getsize(path) == fixture_size:
                continue
            
            cid = f.replace('.Loxone', '')
            case = load_case(cid)
            if not case:
                continue
            
            config_info = get_config_diff(str(FIXTURE), path)
            prompt = build_judge_prompt(case, config_info)
            
            results.append({
                'case_id': cid,
                'utterance': case['utterance'],
                'difficulty': case.get('difficulty', '?'),
                'prompt': prompt,
                'config_check': config_info['check'].strip(),
            })
        
        # Write prompts for external LLM evaluation
        with open(args.output, 'w') as f:
            json.dump(results, f, indent=2, ensure_ascii=False)
        
        print(f"Wrote {len(results)} judge prompts to {args.output}")
        print("Run these through an LLM to get semantic scores.")
    
    elif args.case_id and args.result:
        case = load_case(args.case_id)
        if not case:
            print(f"Case '{args.case_id}' not found", file=sys.stderr)
            sys.exit(1)
        
        config_info = get_config_diff(str(FIXTURE), args.result)
        prompt = build_judge_prompt(case, config_info)
        print(prompt)
    
    else:
        parser.print_help()


if __name__ == '__main__':
    main()
