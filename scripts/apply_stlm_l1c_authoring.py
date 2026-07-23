from __future__ import annotations

import base64
import gzip
from pathlib import Path

BLOBS = {'.github/workflows/stlm-l1c-shadow-ci.yml': 'H4sIAAq9YWoC/71X3W/bNhB/919x8MOyDZNTZ3vSkCKdk2IG0qWIi+wlAEFRJ4kNRWokZcdD/vgdJcVWFGf5cFEgEUzyePe776PmJcaw+HL+Cc6n0QwWBU/NCi4Sh3bJvTQaZvPRyOh4BFDVSjGL/9TofFgDJJZrUaBrVwARlFzqZlFxX/T2x0omh7ySE+vGDzfp//HmjNvcTLwp1eDAeVUyNRXMNUgf38RbXlYK3ZCSVdYk+JDeWbFb0qkRN2gzqbC3idrbdWWk9hNX9PZTIxyJrdDKkmjcYTAnO5/O2OLPD6cXf7OLPxZnl1cfvswv/mI/T8q0d3eSS1/UyeHK2JtMmVULOyLYUQs7EnKybrHd07BUOrKuKOLRiISW0jnyU2NrYbQPEGKwyNPR6KtJmv0B09YtttYuIs9CndTa15HinhzbHHnSxNQ+KqWuaTOGX9+NmgPnsep5VTfhMytQ3ADRk9TKOOmNXXckALUL97kIseQORSAlypPlb6MBl7l2nisFl3UHYns79UZpvj60dBR5Y5QoKMxOiD5RuKFdkSnjzSoYo6yM7sxBN7PS/wJCyapaD2XPOAGDJhQGohcrygKNZStbBLqT5dFopwWcMBWmkBlbck+38g0vMnUMdz1sHR6IIkxlk2ZH746mtGwMBNtkedEdozOZg7uRFROFVKlFfextjQ/4tXn2cgzXPVKA3bm3g+jZ/OvdoOCHVGbZvcxHZiUPUg62pYmYW27XwHUKDa+nrSuCI6FVI6ooaLklEcSAvhlyX1t0w5wIBCHr02f5dBrCTgUHFtlbHkukJtLw7cT3GHIqURZJsSVu2Q1seE716j4yZcAdalRT2Z+2n0OKCgOVrDDjUvVOYni/VaTNpYky+WMdmqPXG/+R+Up0jucYtUl17ApjPRy9/2EKd+CR1OZvw/ONnfh9cL86GL49LJlBTlUeojM46KrUNSX13Y76sNn//5LQkG068XXTig8GwHdg+h18gfoBFYrCwHg7zHRRP2ttmEmdUk2GFD0Kj5su3N29pVo07W1lcphJl7Vu+YYua6mjuecqUOine+fAwGhxTIT0F3hHvgh93h1Pd2HNrPkXO8jPFMynE75VhC58t9xpA3E4BoXJwvrJV2f6Pm9D8SMcjLfz1zhuY6CZwK7OLucf52enbP7p8+XF1XzRTmHtYDY+eL0YJXNJQwcz2wmZCXJCCKgYQtN9PVMKTKyQPtozQe1NpsSOLUmfTO7BltwkPM3rleLrNzPRWFOsK9bEvFE0exKufA9UFkXQU3Gd11SJaGChX5gyhwqbIXEPzo7mPYcsWVNyUDRicNEeULe+8HjrGSeXE3JKK4p6THO0e3h8E0i4DEylM2qvGOK1p1ou/ZolptYpDUpMKOP24KgNo7QPLwG2sazUmapRC3wz1zyYs+LupcgGde1z59S2quFSpgHNaNud6KmhVnztfvzpiRdIXSnD04hbLzPaCw+Rpx4Qrcghul7m92jDc/dhTYUX2gae7biRNlF4izpq2+TbGGSujcX+KI/h2UeIopR0b55r/wFkv/Sz2g8AAA==',
 'docs/experiments/STLM_L1C_SHADOW_OBSERVATION_PREREGISTRATION.md': 'H4sIAAq9YWoC/4VWwXLbNhC98yswk0s7I6mx26atb07S1O64tSfy+GwQWFGYgAALgIrpP+rP9Jv6FhApSk4mF0mkFrvYt+894JVY39/8JW7Olu/EAwWzMaTFddsFvzNRJuOdWG+l9p/FbR0p7Mqru0CBGhNTyM9V9eqVWCeZ+lhV03/40qKmjQ8k0pbExoSYhPJtZylRKWn4oSWXSt7Qu1VOdicDXgraGU1OUVWV6Ch6Z736hMTe2UHUQ87cyRiNa6ZW3ootWb30fRKBYm/TRVUtxdnrw2vl3Y5CaVBaERU5GYyPHPbmtfCb/GWcpo7w4RKK7UZ4IllSvPIknJ6k4oqdlYNoZVJb4ohnCl5spLW1VJ/GZ0uNVAOCW9KmNG9Jao6fEMLeOxQBeh6/scW0tNI1vWxISKdFa1TwEdAhIpBMjGNOYH3ENgFmMm0Jvbq/vxOyT1sfTBoynG+FNlo4n8b3z8TjyIPH6kRPiQvXvI/UB0d6Ja5nwQjZYiC17wEQQMksWfoZS5Cr8xHw8qzKXK+GzmNi0YAo95jciOnSHFGuIOyDUNIBVlI9eqyxTDONMLFc//Hudn0vflBbmR4nsOKC6+pe0dfGx/hi2BF1WkpSyyQX++U6d7zv6BTyDjGxD7TIiPITOi3UHhE6jKweEsXCiLH3D8E/kzulfK4mw1DwyDQvGFIAh4a8C4MyjGGmMdejJ+iLGf/fvw9nyw/nIg1dJmYrMXMlauS0hOCxF4sVCpOogQnWMUvKEuMSC407iuSiSWYHgggra7IcJcWmbHvsbMmd8RAaCl3A8hykwdfQGsfbUgIZMfoBGVFBA/cdvpGnxRtp8XvEnddiwN0WgARsb5Z2mQV+OgMYDrsBQzVHKQMTeNQh49ME2bYyLHfnwlGfODUUj01ZxI18SgXvy2+REEu+wKTiPCWGSQUEDVqivYDhX/NuwEpLrklb/NAGSDLSgLNwSZuGYpau7LjMNLYRJ6Ylg6wxRK725/r27xtk1Mj/Eg0W9R6Rwp1MUfmZhdF2SUBXvcM8UzBl7zMznK3iFTb/mHwInpADmO2cpsMyw6Y+dj+GSAs+iMePxYIuLlij333/yGuyFR3rBAvaPvvYgzeK+CSByjKMLk+ivAAlDW2AGUbprW8GyBZmDpQXAMhb/PPu6vLjH78vuA6Myjvf+j4KObm1r5M0TjTkMt/2++cjZu6OB7ECMRigkvCVgvPJIVb+b3u809SiBp+IBPDPVpnZ1jSmtrQXG+1Ya3t7il8youp89RVeCVjPDtOInHfGx5G9AS93XpUh/rgSfHCmrHnjuj4tshgXszOjUC8Lajyz9nZV/VQ2caitUTiAo0XE31DYz2j9hXBnWjgcVgK0cJz2qGU0umGVv1nNvP7EWHFu8hynJqtfVrPtcmOj4+VzjNUooRCU3NtQMPvERUS8Xx9w/P66KvhOkyszK/UibiDesgKr3wpEm97aA3MmM5/iy1GcGfWelInlkgNrrh7vLtdrPrb+6U3gowIID6PZHmjHyufrzUpcOlgOxOR5mCwMtRg3v5HG5oOpTHN6hAqmu8r4tpAV2cdWDjM9Fnsn0xYH12Uuz0ppTYrl2iWRFszFajx0R/c9GOp7Yb1rTOq1yXercjEYr3Ki26KtfJXQHn0fX0CKtfDJdnoVybt8YSiLYzuBA9yjYSYnvODmuhgBgONLRygN7um1qv4HRrq4NfsKAAA=',
 'docs/experiments/STLM_L1C_SHADOW_OBSERVATION_STATUS.md': 'H4sIAAq9YWoC/21US27bMBTc6xQPSJdRgnTZnWE7rQHHCWy1Xcq0+CSzpUiVHzfOjXqZnqlDyhHSohtB0vvNmxnyinbV+oHWd+Wcdkch7U96PHh2JxGUNbQLIkRfFFdXNNfCe9WqJkeKYr96eFovH5abalatHjf103KzWG0+1ver7a6q548pWi3r7efNPtfvGjtwUeRJQkpPwpAdQqkMDdaH0rEfrPFMNs9nR611FI5MeMdcduVBNN9ZkuoHZ0/KjxA9a26CdTdUIRcRGZscmBoqT60yQqsXFB8YbV+H5A5j4TTVccPqxMBH4Tyg4vevL3fl/Xs6RCM1A7ak1tkXfjMB7Tt2g1MmXCfIKdRYhyUb2w/CKQ88PQchRRD0U4WjjQE5ITqDUsKiAzuvfEhfDUYoZDIFfg43mb1tNEH1jH6mVV10FxFKmmOAAqyWoZTjD7T3QfelvmtKn/XcI+m12mN0c0TOrpptIdSyTurX67t5vfs0Wzx+TclrlliGLMhwSvJ/s9fLxcfltn6aVZ9SyYJbEXUgnUtR8W4qWcyq2a22nb9NuGrgqkdcN99AioY3/pEN+IVrleNSQ4b962KZSSzqsyVe1bqGKiH/cX+tSHJEhGxLElY5ANqFSf4R0V0Sn7CdaXhEgIk+5CGawfx8lTpSH/EzuY0/JLKVkTwwHia8UWn05+VclMTPokniDlqc8Wk4Bid0CeWCsxpokN/lwWU2mwmlFqaLouPJUTk0vpaHc0gE8WRYBKfhZbIICdCBhsmXfWZjFCIjZq06hfVLPiXYsKKeumjrQYSI8KNTIaFNKAWKHIhVWl5TUiPE4Tr7/shChyP52DTsx2vhydkO2HyS7mBxRIQ7F8WMhnRbwMwqEdpj8nhaYfNeQRZIf063zoK0NZ0KUaYTSqM1II3Q8XI4V4GkhezGhkv1dAEAe14fGivT6pg5SK6ZyPM3xR+zVUoB4gQAAA==',
 'lib/examples/stlm_l1c_shadow_probe.rs': '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',
 'lib/stlm_l1c_shadow.rs': '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[... ELLIPSIZATION ...]dhbv9GXXVYDaLEIw96JR6yCLKbj8vH37qIMQ0AelDIffzzEzKlpkM7OwcGX3+uUBbirEkp0n9LSecnNx6ww3Y4SP5w+Up2wKzaeY8fTZbK9N8kqOIY1jvBic0JmQbFPNtEIZSFZ1CXSgXjHanvf/K+sPvwhxfKVpiwFFkV/mbLuoIMxL210F/Mrhk/3pD+aFBRwZOxjWGdnRFUxHt2/3p/t9y0lJBVdH6x+OjyZ920HLDmptXcCyAkmjNzCCTo8wEdfkQ5GX92xt/pInmU8vBO4Pbb22+aDR4PfHVmIZ2jS/KPuut477oLULG5gErKgKswcfPQrEbz02b7ewxFawRq5+uq3UPlEEvL5BrXRCIKx0kjWZA0dcCHGqrp/rbmZOrOJkGIlNnkRiSrtVSqUqaIyRdhgY2k9bGV6RuElUfXlVCIZT5ULrkm0stKXj5z5wY2Nd5V84Xub4rje0ge3yV6ufx/WhrFl4svXiBkPB6gDMi8Yb+huX2SwIxAeK1yw3VSpa5frvEUWhal99e6A2Y5+5oFvEvr2DazLoT27i8/l2+I+YPAtfPtuGXfLT7echNGjYQNF5iehRVFXEl/CFLxD2SDXuHP9/82JCupi+RmDFA2to4LsmnIkllvIs6tR/crMFm17c+MI/W+dPh6phZw04I6xhcYXTmpTwGgq8/hP9zCr6RPytF2pViIEV1z67JQ0vm2fc3HohAnRbLxRlAWCaksCXVEnQ23fi+QXIALigH3uLfObhx1KJ85X1Es/lf71jRv4sfx+D7aDpPpSF51qN+PVU8EHeT9XEcOiDA96QzXLm6XdkcSvbOtShzHCLCH+g4zbjRUP1nhtFpbO+fLmLdd7z0izzfHZrr0/W7Ixaq9w3/tJMU3DM/nYJ+5su9K/FNH8opdHnfev9Qa0SZ0GdXhhsGQ5dDr+BXOE42lEQ/KGvRioqjXtffUDx+Sg4abm4jA+LX2DmtznIr6/etV6KLQnBe8sKUIPi3dh9y3VXK+eN112NVt1+3dXWfRN2V6vVt2J3te64Iburmfu2bEurjaaZ9ersx7RQbnXubNa6UnsltO167e5G3ddhd7Z1XI3d2abzYuvOlq6rpjsbddwlPdzpWNfqxY7sNsYST340qbsy1HN6rWXODv/ttpS7rrD8UNc0yiGMtm4rqMVun4SkmJuCexWfNTAJ2xIZV9M2Bng1JtjhPutK/YNieH9dU4LZNygrYdFkMzdpX4Ik3sfaNNEhKKoH733BR+pB9aZMV3TSnOOWQPax6ojZs9+DmcltSH7d8E0qRba63YuofnOjUcgctSm3rBuz1diVuF6nbfeB5zUQdNe8roPBeWq5a+nziEcTXm2MQ5arpinHHD/PZNCQ6WcpAIFiXkQXWMNRNtnfgXl8+DFnfY1yGxmiVnMv7dd8E9p+EadRsmgf4Oa7j57jlD82uvforzGrMQflAF/BW27On1r6YtuBIHiFN3qHe3vmJZQrzlGHlAO08mADddkNWGfCtMU5aaZDvTDg4bOVsnEtzh+i+h6hh1fGW8WtFg1yPgCKt2nft6ytlqD2VgnCGAOS9wL+3ttQApticfni9XBoM+O2Fv2OrUR/2LmyzYuR1cMypeJa4B+FxCtiqDPVuDV/pYnv6ZexqfZee2U9O4M/3ZInYZVd1md6A+tY3kcYXHFG0zqAVl3KFYk3PrTMQOfBpo+jR2GwnFL62Mk+NgDKsiuQZUTyOx0oy11kn+0nrefPPMqNMW6tedxstMzBxm6bxkeiuL/3Hjeh1itNVhDShcbCojaqFa4WaLSH3v8BoBfhZgd4AAA='}

PATCHES = [('lib/lib.rs',
  '// ΩV1-F2: post-response learned-expression shadow observation. The live HTTP\n'
  '// response is frozen first; only typed intent-derived semantics, sealed VoiceState\n'
  '// projection data, bounded fingerprints, and metadata enter the isolated worker.\n'
  '#[cfg(feature = "omega-v1-f2-shadow")]\n'
  'pub mod omega_v1f2_shadow;\n',
  '// ΩV1-F2: post-response learned-expression shadow observation. The live HTTP\n'
  '// response is frozen first; only typed intent-derived semantics, sealed VoiceState\n'
  '// projection data, bounded fingerprints, and metadata enter the isolated worker.\n'
  '#[cfg(feature = "omega-v1-f2-shadow")]\n'
  'pub mod omega_v1f2_shadow;\n'
  '\n'
  '// STLM L1-C: post-response shadow comparison for verifier-backed improvisation.\n'
  '// It consumes only the sealed F2 bundle and response fingerprint, records bounded\n'
  '// metadata without candidate text, and has no live response or state authority.\n'
  '#[cfg(feature = "stlm-l1c-shadow")]\n'
  'pub mod stlm_l1c_shadow;\n'),
 ('lib/Cargo.toml',
  '[[example]]\n'
  'name = "stlm_l1b_heldout_conversation_evaluation"\n'
  'path = "examples/stlm_l1b_heldout_conversation_evaluation.rs"\n'
  'required-features = ["verified-improvisation"]\n',
  '[[example]]\n'
  'name = "stlm_l1b_heldout_conversation_evaluation"\n'
  'path = "examples/stlm_l1b_heldout_conversation_evaluation.rs"\n'
  'required-features = ["verified-improvisation"]\n'
  '\n'
  '[[example]]\n'
  'name = "stlm_l1c_shadow_probe"\n'
  'path = "examples/stlm_l1c_shadow_probe.rs"\n'
  'required-features = ["stlm-l1c-shadow"]\n'),
 ('lib/Cargo.toml',
  'verified-improvisation = ["omega-v1-learned-expression"]  # STLM L1-A verifier-backed candidate search; '
  'offline-only\n'
  'omega-v1-f2-shadow = ["omega-v1-http-canary", "omega-v1-learned-expression"]  # ΩV1-F2 post-response shadow '
  'observation; no learned text return\n',
  'verified-improvisation = ["omega-v1-learned-expression"]  # STLM L1-A verifier-backed candidate search; '
  'offline-only\n'
  'omega-v1-f2-shadow = ["omega-v1-http-canary", "omega-v1-learned-expression"]  # ΩV1-F2 post-response shadow '
  'observation; no learned text return\n'
  'stlm-l1c-shadow = ["verified-improvisation", "omega-v1-f2-shadow"]  # STLM L1-C verified-improvisation metadata '
  'observation; no candidate text return\n'),
 ('src/Cargo.toml',
  "# Production enables the library's F2 observer but intentionally does not enable\n"
  "# the binary crate's legacy HTTP proxy cfg. Runtime::chat remains the text authority.\n"
  'starfire-live = ["star/omega-v1-f2-shadow"]\n',
  "# Production enables the library's F2 and L1-C observers but intentionally does\n"
  "# not enable the binary crate's legacy HTTP proxy cfg. Runtime::chat remains the text authority.\n"
  'starfire-live = ["star/stlm-l1c-shadow"]\n'),
 ('entrypoint.sh',
  '# ΩV1-F2 is compiled into the binary but remains inert until this explicit\n'
  '# switch is enabled after the external build/deploy gate succeeds.\n'
  'export STARFIRE_OMEGA_V1F2_SHADOW="${STARFIRE_OMEGA_V1F2_SHADOW:-0}"\n',
  '# ΩV1-F2 and STLM L1-C are compiled into the binary but remain inert until\n'
  '# their explicit switches are enabled after the external build/deploy gates succeed.\n'
  'export STARFIRE_OMEGA_V1F2_SHADOW="${STARFIRE_OMEGA_V1F2_SHADOW:-0}"\n'
  'export STARFIRE_STLM_L1C_SHADOW="${STARFIRE_STLM_L1C_SHADOW:-0}"\n'),
 ('lib/api.rs',
  '    #[cfg(feature = "omega-v1-f2-shadow")]\n'
  '    let shadow_event = if crate::omega_v1f2_shadow::shadow_enabled() {\n'
  '        Some(crate::omega_v1f2_shadow::event_from_intent(\n'
  '            &crate::runtime::response_intent::classify(&req.message),\n'
  '        ))\n'
  '    } else {\n'
  '        None\n'
  '    };\n',
  '    #[cfg(feature = "omega-v1-f2-shadow")]\n'
  '    let omega_v1f2_shadow_enabled = crate::omega_v1f2_shadow::shadow_enabled();\n'
  '    #[cfg(feature = "stlm-l1c-shadow")]\n'
  '    let stlm_l1c_shadow_enabled = crate::stlm_l1c_shadow::shadow_enabled();\n'
  '    #[cfg(feature = "omega-v1-f2-shadow")]\n'
  '    let shadow_event = {\n'
  '        let mut enabled = omega_v1f2_shadow_enabled;\n'
  '        #[cfg(feature = "stlm-l1c-shadow")]\n'
  '        {\n'
  '            enabled |= stlm_l1c_shadow_enabled;\n'
  '        }\n'
  '        if enabled {\n'
  '            Some(crate::omega_v1f2_shadow::event_from_intent(\n'
  '                &crate::runtime::response_intent::classify(&req.message),\n'
  '            ))\n'
  '        } else {\n'
  '            None\n'
  '        }\n'
  '    };\n'),
 ('lib/api.rs',
  '            #[cfg(feature = "omega-v1-f2-shadow")]\n'
  '            if let Some(shadow_event) = shadow_event {\n'
  '                let fingerprint =\n'
  '                    crate::omega_v1f2_shadow::ResponseFingerprint::frozen(response_json.as_bytes());\n'
  '                crate::omega_v1f2_shadow::dispatch(shadow_event, fingerprint);\n'
  '            }\n',
  '            #[cfg(feature = "omega-v1-f2-shadow")]\n'
  '            if let Some(shadow_event) = shadow_event {\n'
  '                let fingerprint =\n'
  '                    crate::omega_v1f2_shadow::ResponseFingerprint::frozen(response_json.as_bytes());\n'
  '                if omega_v1f2_shadow_enabled {\n'
  '                    crate::omega_v1f2_shadow::dispatch(shadow_event.clone(), fingerprint);\n'
  '                }\n'
  '                #[cfg(feature = "stlm-l1c-shadow")]\n'
  '                if stlm_l1c_shadow_enabled {\n'
  '                    crate::stlm_l1c_shadow::dispatch(shadow_event, fingerprint);\n'
  '                }\n'
  '            }\n'),
 ('Dockerfile',
  '# Build the production executable with an explicit live-integration opt-in.\n'
  '# The F2 shadow worker remains a dependency of starfire-live, not its synonym.\n'
  'RUN cargo build --release --locked -p star_bin --bin star --features starfire-live\n',
  '# STLM L1-C builder gate. The verified-improvisation candidate is independently\n'
  '# checked, compared with the neutral control, reduced to metadata, and prohibited\n'
  '# from altering or entering the finalized response bytes.\n'
  'RUN cargo test -p star --lib --features stlm-l1c-shadow --locked \\\n'
  '        stlm_l1c_shadow:: -- --test-threads=1 \\\n'
  '    && cargo run -p star --example stlm_l1c_shadow_probe \\\n'
  '        --features stlm-l1c-shadow --locked \\\n'
  '        | tee /tmp/stlm-l1c-shadow-report.json \\\n'
  '    && grep -F '\''"experiment": "STLM_L1C_VERIFIED_IMPROVISATION_SHADOW"'\'' /tmp/stlm-l1c-shadow-report.json \\\n'
  '    && grep -F '\''"independent_candidate_verified": true'\'' /tmp/stlm-l1c-shadow-report.json \\\n'
  '    && grep -F '\''"exact_replay": true'\'' /tmp/stlm-l1c-shadow-report.json \\\n'
  '    && grep -F '\''"response_bytes_preserved": true'\'' /tmp/stlm-l1c-shadow-report.json \\\n'
  '    && grep -F '\''"candidate_text_absent_from_ledger": true'\'' /tmp/stlm-l1c-shadow-report.json \\\n'
  '    && grep -F '\''"authority_boundary_closed": true'\'' /tmp/stlm-l1c-shadow-report.json \\\n'
  '    && grep -F '\''"no_runtime_response_influence": true'\'' /tmp/stlm-l1c-shadow-report.json \\\n'
  '    && grep -F '\''"gate_passed": true'\'' /tmp/stlm-l1c-shadow-report.json\n'
  '\n'
  '# Build the production executable with explicit live-integration opt-ins.\n'
  '# F2 and L1-C remain compiled observers, while Runtime::chat remains text authority.\n'
  'RUN cargo build --release --locked -p star_bin --bin star --features starfire-live\n')]

def write_blobs() -> None:
    for name, encoded in BLOBS.items():
        path = Path(name)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(gzip.decompress(base64.b64decode(encoded)))

def replace_once(path_name: str, old: str, new: str) -> None:
    path = Path(path_name)
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one replacement in {path_name}, found {count}")
    path.write_text(text.replace(old, new, 1))

def main() -> None:
    write_blobs()
    for path_name, old, new in PATCHES:
        replace_once(path_name, old, new)

if __name__ == "__main__":
    main()
