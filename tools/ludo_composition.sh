#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# ludo_composition.sh — ludoSpring interactive game science composition
#
# Exercises ludoSpring's exploration lane: interaction fidelity, real-time
# feedback, multi-input routing, DAG branching, and braid provenance.
#
# Built on nucleus_composition_lib.sh from primalSpring.
# See tools/ttt_composition.sh in primalSpring for the reference implementation.
#
# Usage:
#   COMPOSITION_NAME=ludo ./tools/ludo_composition.sh
#   FAMILY_ID=ludo ./tools/ludo_composition.sh

set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────

COMPOSITION_NAME="${COMPOSITION_NAME:-ludo}"
REQUIRED_CAPS="visualization security"
OPTIONAL_CAPS="compute tensor dag ledger attribution"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/nucleus_composition_lib.sh"

# ── Game State ───────────────────────────────────────────────────────

RUNNING=true
MODE="menu"          # menu | fitts | reaction | explore
TRIAL_NUM=0
TRIAL_TOTAL=10
TRIAL_START_MS=0
RESULTS=()
TARGET_X=0
TARGET_Y=0
TARGET_W=60
REACTION_VISIBLE=false
REACTION_WAIT_MS=0

GRID_X0=40
GRID_Y0=80
CANVAS_W=420
CANVAS_H=360

# ── Hit Testing ──────────────────────────────────────────────────────

hit_test_fn() {
    local px="$1" py="$2"
    px="${px%.*}"
    py="${py%.*}"

    case "$MODE" in
        menu)
            if (( py >= 160 && py < 200 )); then echo 1; return; fi
            if (( py >= 210 && py < 250 )); then echo 2; return; fi
            if (( py >= 260 && py < 300 )); then echo 3; return; fi
            ;;
        fitts)
            local dx=$(( px - TARGET_X ))
            local dy=$(( py - TARGET_Y ))
            local r=$(( TARGET_W / 2 ))
            if (( dx*dx + dy*dy <= r*r )); then echo 0; return; fi
            ;;
        reaction)
            if (( px >= GRID_X0 && px < GRID_X0 + CANVAS_W && py >= GRID_Y0 && py < GRID_Y0 + CANVAS_H )); then
                echo 0; return
            fi
            ;;
    esac
    echo -1
}

# ── Timing ───────────────────────────────────────────────────────────

now_ms() {
    echo $(($(date +%s%N) / 1000000))
}

# ── Fitts Target Placement (random within canvas) ────────────────────

place_fitts_target() {
    local margin=$((TARGET_W / 2 + 10))
    TARGET_X=$(( GRID_X0 + margin + RANDOM % (CANVAS_W - 2*margin) ))
    TARGET_Y=$(( GRID_Y0 + margin + RANDOM % (CANVAS_H - 2*margin) ))
}

# ── Science: Compute Fitts ID via barraCuda ──────────────────────────

compute_fitts_id() {
    local distance="$1" width="$2"
    if cap_available tensor; then
        local resp
        resp=$(send_rpc "$(cap_socket tensor)" "activation.fitts" \
            "{\"distance\":$distance,\"width\":$width}")
        local id_val
        id_val=$(echo "$resp" | grep -oP '"index_of_difficulty"\s*:\s*\K[0-9.]+' | head -1 || true)
        if [[ -n "$id_val" ]]; then
            echo "$id_val"
            return
        fi
    fi
    echo "?"
}

compute_hick_bits() {
    local n="$1"
    if cap_available tensor; then
        local resp
        resp=$(send_rpc "$(cap_socket tensor)" "activation.hick" \
            "{\"n_choices\":$n}")
        local bits
        bits=$(echo "$resp" | grep -oP '"information_bits"\s*:\s*\K[0-9.]+' | head -1 || true)
        if [[ -n "$bits" ]]; then
            echo "$bits"
            return
        fi
    fi
    echo "?"
}

# ── Rendering ────────────────────────────────────────────────────────

render_menu() {
    local title
    title=$(make_text_node "title" 230 40 "NUCLEUS ludoSpring" 28 0.95 0.95 1.0)
    local sub
    sub=$(make_text_node "sub" 230 75 "Interactive Game Science Composition" 14 0.5 0.7 0.5)

    local opt1
    opt1=$(make_text_node "opt1" 230 175 "[1] Fitts Law — Pointing Task (click targets)" 16 0.8 0.85 0.9)
    local opt2
    opt2=$(make_text_node "opt2" 230 225 "[2] Reaction Time — Hick's Law (click when green)" 16 0.8 0.85 0.9)
    local opt3
    opt3=$(make_text_node "opt3" 230 275 "[3] Free Explore — DAG branching sandbox" 16 0.8 0.85 0.9)

    local info_text="[NUCLEUS]"
    for cap in $REQUIRED_CAPS $OPTIONAL_CAPS; do
        if cap_available "$cap"; then
            info_text="$info_text $cap:✓"
        else
            info_text="$info_text $cap:✗"
        fi
    done
    local info
    info=$(make_text_node "info" 230 350 "$info_text" 10 0.45 0.55 0.45)

    local keys
    keys=$(make_text_node "keys" 230 400 "Press 1-3 or click | Q to quit" 12 0.4 0.4 0.5)

    local root
    root=$(printf '"root":{"id":"root","transform":{"a":1.0,"b":0.0,"c":0.0,"d":1.0,"tx":0.0,"ty":0.0},"primitives":[],"children":["title","sub","opt1","opt2","opt3","info","keys"],"visible":true,"opacity":1.0,"label":null,"data_source":null}')
    local scene="{\"nodes\":{${root},${title},${sub},${opt1},${opt2},${opt3},${info},${keys}},\"root_id\":\"root\"}"
    push_scene "ludo-main" "$scene"
}

render_fitts_trial() {
    local status="${1:-Click the green target}"
    local trial_info="Trial $((TRIAL_NUM + 1))/$TRIAL_TOTAL"

    local title
    title=$(make_text_node "ft-title" 230 20 "Fitts Law — Pointing Task" 22 0.95 0.95 1.0)
    local trial_label
    trial_label=$(make_text_node "ft-trial" 230 48 "$trial_info" 13 0.6 0.7 0.6)
    local status_node
    status_node=$(make_text_node "ft-status" 230 430 "$status" 14 0.7 0.7 0.8)

    local target_node
    local tx=$((TARGET_X - TARGET_W/2))
    local ty=$((TARGET_Y - TARGET_W/2))
    target_node="\"ft-target\":{\"id\":\"ft-target\",\"transform\":{\"a\":1.0,\"b\":0.0,\"c\":0.0,\"d\":1.0,\"tx\":0.0,\"ty\":0.0},\"primitives\":[{\"Rect\":{\"x\":${tx}.0,\"y\":${ty}.0,\"width\":${TARGET_W}.0,\"height\":${TARGET_W}.0,\"fill\":{\"r\":0.2,\"g\":0.85,\"b\":0.35,\"a\":0.9},\"stroke\":null,\"corner_radius\":${TARGET_W}.0,\"data_id\":\"ft-target\"}}],\"children\":[],\"visible\":true,\"opacity\":1.0,\"label\":\"target\",\"data_source\":null}"

    local root
    root=$(printf '"root":{"id":"root","transform":{"a":1.0,"b":0.0,"c":0.0,"d":1.0,"tx":0.0,"ty":0.0},"primitives":[],"children":["ft-title","ft-trial","ft-target","ft-status"],"visible":true,"opacity":1.0,"label":null,"data_source":null}')
    local scene="{\"nodes\":{${root},${title},${trial_label},${target_node},${status_node}},\"root_id\":\"root\"}"
    push_scene "ludo-main" "$scene"
}

render_reaction_wait() {
    local title
    title=$(make_text_node "rx-title" 230 20 "Reaction Time — Hick's Law" 22 0.95 0.95 1.0)
    local prompt
    prompt=$(make_text_node "rx-prompt" 230 220 "Wait for green..." 24 0.8 0.4 0.4)

    local bg="\"rx-bg\":{\"id\":\"rx-bg\",\"transform\":{\"a\":1.0,\"b\":0.0,\"c\":0.0,\"d\":1.0,\"tx\":0.0,\"ty\":0.0},\"primitives\":[{\"Rect\":{\"x\":${GRID_X0}.0,\"y\":${GRID_Y0}.0,\"width\":${CANVAS_W}.0,\"height\":${CANVAS_H}.0,\"fill\":{\"r\":0.25,\"g\":0.12,\"b\":0.12,\"a\":0.6},\"stroke\":null,\"corner_radius\":12.0,\"data_id\":\"rx-bg\"}}],\"children\":[],\"visible\":true,\"opacity\":1.0,\"label\":null,\"data_source\":null}"

    local root
    root=$(printf '"root":{"id":"root","transform":{"a":1.0,"b":0.0,"c":0.0,"d":1.0,"tx":0.0,"ty":0.0},"primitives":[],"children":["rx-bg","rx-title","rx-prompt"],"visible":true,"opacity":1.0,"label":null,"data_source":null}')
    local scene="{\"nodes\":{${root},${bg},${title},${prompt}},\"root_id\":\"root\"}"
    push_scene "ludo-main" "$scene"
}

render_reaction_go() {
    local title
    title=$(make_text_node "rx-title" 230 20 "Reaction Time — Hick's Law" 22 0.95 0.95 1.0)
    local prompt
    prompt=$(make_text_node "rx-prompt" 230 220 "CLICK NOW!" 36 0.2 0.95 0.3)
    local trial_info="Trial $((TRIAL_NUM + 1))/$TRIAL_TOTAL"
    local trial_label
    trial_label=$(make_text_node "rx-trial" 230 48 "$trial_info" 13 0.6 0.7 0.6)

    local bg="\"rx-bg\":{\"id\":\"rx-bg\",\"transform\":{\"a\":1.0,\"b\":0.0,\"c\":0.0,\"d\":1.0,\"tx\":0.0,\"ty\":0.0},\"primitives\":[{\"Rect\":{\"x\":${GRID_X0}.0,\"y\":${GRID_Y0}.0,\"width\":${CANVAS_W}.0,\"height\":${CANVAS_H}.0,\"fill\":{\"r\":0.1,\"g\":0.35,\"b\":0.12,\"a\":0.7},\"stroke\":null,\"corner_radius\":12.0,\"data_id\":\"rx-bg\"}}],\"children\":[],\"visible\":true,\"opacity\":1.0,\"label\":null,\"data_source\":null}"

    local root
    root=$(printf '"root":{"id":"root","transform":{"a":1.0,"b":0.0,"c":0.0,"d":1.0,"tx":0.0,"ty":0.0},"primitives":[],"children":["rx-bg","rx-title","rx-trial","rx-prompt"],"visible":true,"opacity":1.0,"label":null,"data_source":null}')
    local scene="{\"nodes\":{${root},${bg},${title},${trial_label},${prompt}},\"root_id\":\"root\"}"
    push_scene "ludo-main" "$scene"
}

render_results() {
    local title_text="$1"
    shift
    local lines=("$@")

    local title
    title=$(make_text_node "res-title" 230 30 "$title_text" 22 0.95 0.95 1.0)

    local all_nodes="${title}"
    local all_ids="\"res-title\""
    local y=80
    local idx=0
    for line in "${lines[@]}"; do
        local nid="res-l${idx}"
        local node
        node=$(make_text_node "$nid" 230 $y "$line" 13 0.75 0.78 0.85)
        all_nodes="${all_nodes},${node}"
        all_ids="${all_ids},\"${nid}\""
        y=$((y + 22))
        idx=$((idx + 1))
    done

    local back
    back=$(make_text_node "res-back" 230 $((y + 30)) "Press M for menu | Q to quit" 14 0.5 0.5 0.6)
    all_nodes="${all_nodes},${back}"
    all_ids="${all_ids},\"res-back\""

    local root
    root=$(printf '"root":{"id":"root","transform":{"a":1.0,"b":0.0,"c":0.0,"d":1.0,"tx":0.0,"ty":0.0},"primitives":[],"children":[%s],"visible":true,"opacity":1.0,"label":null,"data_source":null}' "$all_ids")
    local scene="{\"nodes\":{${root},${all_nodes}},\"root_id\":\"root\"}"
    push_scene "ludo-main" "$scene"
}

# ── Mode: Fitts Law Pointing Task ────────────────────────────────────

start_fitts() {
    MODE="fitts"
    TRIAL_NUM=0
    RESULTS=()
    TARGET_W=60
    dag_create_session "ludo-fitts" "[{\"key\":\"task\",\"value\":\"fitts\"}]"
    ledger_create_spine "ludo-fitts"
    place_fitts_target
    TRIAL_START_MS=$(now_ms)
    render_fitts_trial "Click the green target"
    ok "fitts task started — $TRIAL_TOTAL trials, target_w=$TARGET_W"
}

handle_fitts_click() {
    local cell="$1"
    if [[ "$cell" -ne 0 ]]; then return; fi

    local elapsed=$(( $(now_ms) - TRIAL_START_MS ))
    RESULTS+=("$elapsed")
    log "fitts trial $((TRIAL_NUM+1)): ${elapsed}ms"

    local fitts_id
    fitts_id=$(compute_fitts_id 200 "$TARGET_W")

    dag_append_event "ludo-fitts" "hit" "$TRIAL_NUM:$elapsed" \
        "[{\"key\":\"trial\",\"value\":\"$TRIAL_NUM\"},{\"key\":\"mt_ms\",\"value\":\"$elapsed\"},{\"key\":\"fitts_id\",\"value\":\"$fitts_id\"}]" \
        "click" "$ACCUMULATED_HOVER_MOVES"
    braid_record "fitts-hit" "application/x-ludo-fitts" "$TRIAL_NUM:$elapsed" \
        "{\"trial\":$TRIAL_NUM,\"mt_ms\":$elapsed,\"fitts_id\":\"$fitts_id\",\"target_w\":$TARGET_W}" \
        "click" "$ACCUMULATED_HOVER_MOVES"
    ACCUMULATED_HOVER_MOVES=0

    TRIAL_NUM=$((TRIAL_NUM + 1))
    if [[ $TRIAL_NUM -ge $TRIAL_TOTAL ]]; then
        finish_fitts
        return
    fi

    TARGET_W=$(( 30 + RANDOM % 70 ))
    place_fitts_target
    TRIAL_START_MS=$(now_ms)
    render_fitts_trial "${elapsed}ms — next target"
}

finish_fitts() {
    local sum=0
    for r in "${RESULTS[@]}"; do sum=$((sum + r)); done
    local avg=$((sum / ${#RESULTS[@]}))
    local min=${RESULTS[0]} max=${RESULTS[0]}
    for r in "${RESULTS[@]}"; do
        (( r < min )) && min=$r
        (( r > max )) && max=$r
    done

    local fitts_id
    fitts_id=$(compute_fitts_id 200 50)

    local lines=(
        "Trials: $TRIAL_TOTAL"
        "Mean MT: ${avg}ms"
        "Min: ${min}ms  Max: ${max}ms"
        "Fitts ID (D=200, W=50): $fitts_id bits"
        ""
        "DAG vertices: ${#VERTEX_STACK[@]}"
        "Braid session: $BRAID_SESSION_TAG"
    )

    ledger_seal_spine
    braid_record "fitts-summary" "application/x-ludo-summary" "avg=${avg}ms" \
        "{\"task\":\"fitts\",\"trials\":$TRIAL_TOTAL,\"avg_ms\":$avg,\"min_ms\":$min,\"max_ms\":$max}" \
        "system" "0"

    ok "fitts complete: avg=${avg}ms min=${min}ms max=${max}ms fitts_id=$fitts_id"
    MODE="results"
    render_results "Fitts Law Results" "${lines[@]}"
}

# ── Mode: Reaction Time (Hick's Law) ────────────────────────────────

start_reaction() {
    MODE="reaction"
    TRIAL_NUM=0
    RESULTS=()
    REACTION_VISIBLE=false
    dag_create_session "ludo-reaction" "[{\"key\":\"task\",\"value\":\"reaction\"}]"
    ledger_create_spine "ludo-reaction"
    schedule_reaction
    ok "reaction task started — $TRIAL_TOTAL trials"
}

schedule_reaction() {
    REACTION_VISIBLE=false
    REACTION_WAIT_MS=$(( 1000 + RANDOM % 3000 ))
    TRIAL_START_MS=$(now_ms)
    render_reaction_wait
}

check_reaction_timer() {
    if $REACTION_VISIBLE; then return; fi
    local elapsed=$(( $(now_ms) - TRIAL_START_MS ))
    if (( elapsed >= REACTION_WAIT_MS )); then
        REACTION_VISIBLE=true
        TRIAL_START_MS=$(now_ms)
        render_reaction_go
    fi
}

handle_reaction_click() {
    if ! $REACTION_VISIBLE; then
        log "false start — too early!"
        render_reaction_wait
        return
    fi

    local elapsed=$(( $(now_ms) - TRIAL_START_MS ))
    RESULTS+=("$elapsed")
    log "reaction trial $((TRIAL_NUM+1)): ${elapsed}ms"

    local hick_bits
    hick_bits=$(compute_hick_bits 2)

    dag_append_event "ludo-reaction" "response" "$TRIAL_NUM:$elapsed" \
        "[{\"key\":\"trial\",\"value\":\"$TRIAL_NUM\"},{\"key\":\"rt_ms\",\"value\":\"$elapsed\"},{\"key\":\"hick_bits\",\"value\":\"$hick_bits\"}]" \
        "click" "$ACCUMULATED_HOVER_MOVES"
    braid_record "reaction-hit" "application/x-ludo-reaction" "$TRIAL_NUM:$elapsed" \
        "{\"trial\":$TRIAL_NUM,\"rt_ms\":$elapsed,\"hick_bits\":\"$hick_bits\"}" \
        "click" "$ACCUMULATED_HOVER_MOVES"
    ACCUMULATED_HOVER_MOVES=0

    TRIAL_NUM=$((TRIAL_NUM + 1))
    if [[ $TRIAL_NUM -ge $TRIAL_TOTAL ]]; then
        finish_reaction
        return
    fi
    schedule_reaction
}

finish_reaction() {
    local sum=0
    for r in "${RESULTS[@]}"; do sum=$((sum + r)); done
    local avg=$((sum / ${#RESULTS[@]}))
    local min=${RESULTS[0]} max=${RESULTS[0]}
    for r in "${RESULTS[@]}"; do
        (( r < min )) && min=$r
        (( r > max )) && max=$r
    done

    local hick_bits
    hick_bits=$(compute_hick_bits 2)

    local lines=(
        "Trials: $TRIAL_TOTAL"
        "Mean RT: ${avg}ms"
        "Min: ${min}ms  Max: ${max}ms"
        "Hick bits (n=2, go/no-go): $hick_bits"
        ""
        "DAG vertices: ${#VERTEX_STACK[@]}"
        "Braid session: $BRAID_SESSION_TAG"
    )

    ledger_seal_spine
    braid_record "reaction-summary" "application/x-ludo-summary" "avg=${avg}ms" \
        "{\"task\":\"reaction\",\"trials\":$TRIAL_TOTAL,\"avg_ms\":$avg,\"min_ms\":$min,\"max_ms\":$max}" \
        "system" "0"

    ok "reaction complete: avg=${avg}ms min=${min}ms max=${max}ms hick_bits=$hick_bits"
    MODE="results"
    render_results "Reaction Time Results" "${lines[@]}"
}

# ── Mode: Free Explore (DAG sandbox) ─────────────────────────────────

EXPLORE_DEPTH=0

start_explore() {
    MODE="explore"
    EXPLORE_DEPTH=0
    dag_create_session "ludo-explore" "[{\"key\":\"task\",\"value\":\"explore\"}]"
    ledger_create_spine "ludo-explore"
    render_explore "Free explore — press keys, click anywhere. U=undo B=branches T=tree"
    ok "explore sandbox started"
}

render_explore() {
    local status="${1:-Exploring...}"
    local dag_info=""
    if [[ -n "$CURRENT_VERTEX" ]]; then
        dag_info="  vertex=${CURRENT_VERTEX:0:8}.. depth=${#VERTEX_STACK[@]}"
    fi

    local title
    title=$(make_text_node "ex-title" 230 30 "ludoSpring — Free Explore" 22 0.95 0.95 1.0)
    local depth_label
    depth_label=$(make_text_node "ex-depth" 230 58 "Events: $EXPLORE_DEPTH${dag_info}" 12 0.5 0.65 0.5)
    local status_node
    status_node=$(make_text_node "ex-status" 230 220 "$status" 16 0.75 0.78 0.85)
    local keys
    keys=$(make_text_node "ex-keys" 230 420 "Any key = event | Click = event | U undo | B branches | T tree | M menu | Q quit" 10 0.4 0.4 0.5)

    local root
    root=$(printf '"root":{"id":"root","transform":{"a":1.0,"b":0.0,"c":0.0,"d":1.0,"tx":0.0,"ty":0.0},"primitives":[],"children":["ex-title","ex-depth","ex-status","ex-keys"],"visible":true,"opacity":1.0,"label":null,"data_source":null}')
    local scene="{\"nodes\":{${root},${title},${depth_label},${status_node},${keys}},\"root_id\":\"root\"}"
    push_scene "ludo-main" "$scene"
}

# ── Main Loop ────────────────────────────────────────────────────────

main() {
    discover_capabilities || { err "Required primals not found. Run: COMPOSITION_NAME=ludo ./tools/composition_nucleus.sh start"; exit 1; }

    composition_startup "NUCLEUS ludoSpring" "Game Science Composition"

    subscribe_interactions "click"
    subscribe_sensor_stream

    render_menu

    while $RUNNING; do
        local sensor_batch
        sensor_batch=$(poll_sensor_stream)
        process_sensor_batch "$sensor_batch"

        ACCUMULATED_HOVER_MOVES=$((ACCUMULATED_HOVER_MOVES + SENSOR_HOVER_MOVES))

        case "$MODE" in
            menu)
                if [[ -n "$SENSOR_KEY" ]]; then
                    case "$SENSOR_KEY" in
                        1|Num1) start_fitts ;;
                        2|Num2) start_reaction ;;
                        3|Num3) start_explore ;;
                        Q|q|Escape) RUNNING=false ;;
                    esac
                elif [[ "$SENSOR_CLICK_CELL" -ge 0 ]]; then
                    case "$SENSOR_CLICK_CELL" in
                        1) start_fitts ;;
                        2) start_reaction ;;
                        3) start_explore ;;
                    esac
                fi
                ;;

            fitts)
                if $SENSOR_HOVER_CHANGED; then
                    render_fitts_trial "Click the green target (hover: $HOVER_CELL)"
                fi
                if [[ "$SENSOR_CLICK_CELL" -ge 0 ]]; then
                    handle_fitts_click "$SENSOR_CLICK_CELL"
                elif [[ -n "$SENSOR_KEY" ]]; then
                    case "$SENSOR_KEY" in
                        M|m) MODE="menu"; render_menu ;;
                        Q|q|Escape) RUNNING=false ;;
                    esac
                fi
                ;;

            reaction)
                check_reaction_timer
                if [[ "$SENSOR_CLICK_CELL" -ge 0 ]] || [[ -n "$SENSOR_KEY" ]]; then
                    handle_reaction_click
                fi
                if [[ -n "$SENSOR_KEY" ]]; then
                    case "$SENSOR_KEY" in
                        M|m) MODE="menu"; render_menu ;;
                        Q|q|Escape) RUNNING=false ;;
                    esac
                fi
                ;;

            explore)
                if [[ -n "$SENSOR_KEY" ]]; then
                    case "$SENSOR_KEY" in
                        U|u)
                            if [[ ${#VERTEX_STACK[@]} -gt 1 ]]; then
                                unset 'VERTEX_STACK[${#VERTEX_STACK[@]}-1]'
                                unset 'STATE_STACK[${#STATE_STACK[@]}-1]'
                                unset 'INPUT_TYPE_STACK[${#INPUT_TYPE_STACK[@]}-1]'
                                unset 'HOVER_COUNT_STACK[${#HOVER_COUNT_STACK[@]}-1]'
                                local top=$((${#VERTEX_STACK[@]} - 1))
                                CURRENT_VERTEX="${VERTEX_STACK[$top]}"
                                EXPLORE_DEPTH=$((EXPLORE_DEPTH > 0 ? EXPLORE_DEPTH - 1 : 0))
                                render_explore "Undo! depth=${#VERTEX_STACK[@]}"
                            else
                                render_explore "Nothing to undo"
                            fi
                            ;;
                        B|b)
                            if cap_available dag && [[ -n "$CURRENT_VERTEX" ]]; then
                                local resp
                                resp=$(dag_get_children "$CURRENT_VERTEX")
                                local count
                                count=$(echo "$resp" | grep -oP '"[a-f0-9]{64}"' | wc -l || echo "0")
                                render_explore "Branches from here: $count"
                            else
                                render_explore "DAG offline — no branches"
                            fi
                            ;;
                        T|t)
                            if cap_available dag && [[ -n "$CURRENT_VERTEX" ]]; then
                                local merkle
                                merkle=$(dag_merkle_root)
                                render_explore "Merkle root: ${merkle:0:24}..."
                            else
                                render_explore "DAG offline — no tree"
                            fi
                            ;;
                        M|m) MODE="menu"; render_menu ;;
                        Q|q|Escape) RUNNING=false ;;
                        *)
                            EXPLORE_DEPTH=$((EXPLORE_DEPTH + 1))
                            dag_append_event "ludo-explore" "keypress" "$EXPLORE_DEPTH" \
                                "[{\"key\":\"key\",\"value\":\"$SENSOR_KEY\"}]" "keyboard" "0"
                            braid_record "keypress" "application/x-ludo-explore" "$EXPLORE_DEPTH" \
                                "{\"key\":\"$SENSOR_KEY\",\"depth\":$EXPLORE_DEPTH}" "keyboard" "0"
                            render_explore "Key: $SENSOR_KEY (depth=$EXPLORE_DEPTH)"
                            ;;
                    esac
                elif [[ "$SENSOR_CLICK_CELL" -ge 0 ]]; then
                    EXPLORE_DEPTH=$((EXPLORE_DEPTH + 1))
                    dag_append_event "ludo-explore" "click" "$EXPLORE_DEPTH" \
                        "[{\"key\":\"target\",\"value\":\"$SENSOR_CLICK_CELL\"}]" "click" "$ACCUMULATED_HOVER_MOVES"
                    braid_record "click" "application/x-ludo-explore" "$EXPLORE_DEPTH" \
                        "{\"target\":$SENSOR_CLICK_CELL,\"depth\":$EXPLORE_DEPTH}" "click" "$ACCUMULATED_HOVER_MOVES"
                    ACCUMULATED_HOVER_MOVES=0
                    render_explore "Click (depth=$EXPLORE_DEPTH)"
                fi
                ;;

            results)
                if [[ -n "$SENSOR_KEY" ]]; then
                    case "$SENSOR_KEY" in
                        M|m) MODE="menu"; render_menu ;;
                        Q|q|Escape) RUNNING=false ;;
                    esac
                fi
                ;;
        esac

        if [[ -z "$SENSOR_KEY" ]] && [[ "$SENSOR_CLICK_CELL" -eq -1 ]]; then
            check_proprioception
            sleep "$POLL_INTERVAL"
        fi
    done

    composition_summary
    composition_teardown "ludo-main"
}

main
