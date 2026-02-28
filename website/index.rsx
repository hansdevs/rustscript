# ──────────────────────────────────────────────────────────
#  RustScript — Landing Page
#  Build:   rustscript build website/index.rsx -o index.html
#  Preview: rustscript preview website/index.rsx
#  Serve:   rustscript serve website/index.rsx
# ──────────────────────────────────────────────────────────

import "lib/theme.rsx"
import "lib/logo.png"
import "components/hero.rsx"
import "components/about.rsx"
import "components/features.rsx"
import "components/demo.rsx"
import "components/pipeline.rsx"
import "components/footer.rsx"

# ── Page ────────────────────────────────────────────────────
page {

    style {
        bg: "#0a0e17"
        fg: "#e5e7eb"
        font: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
        pad: "0"
    }

    # ════════════════════════════════════════════════════════
    #  NAV BAR
    # ════════════════════════════════════════════════════════
    div {
        style {
            display: "flex"
            justify-content: "space-between"
            align-items: "center"
            pad: "16px 40px"
            border-bottom: "1px solid #1f2937"
            bg: "rgba(10, 14, 23, 0.95)"
            position: "sticky"
            top: "0"
            z-index: "100"
            backdrop-filter: "blur(12px)"
        }

        div {
            style { display: "flex" align-items: "center" gap: "12px" }

            img {
                src: "{logo}"
                alt: "RustScript"
                style { h: "32px" w: "auto" }
            }
            span "RustScript" {
                style {
                    fg: "#f97316"
                    size: "1.1rem"
                    weight: "700"
                }
            }
        }

        div {
            style { display: "flex" gap: "24px" align-items: "center" }

            a "Home" {
                href: "index.html"
                style { fg: "#f97316" text-decoration: "none" size: "0.9rem" weight: "600" }
            }
            a "Compiler" {
                href: "compiler.html"
                style { fg: "#9ca3af" text-decoration: "none" size: "0.9rem" }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  HERO
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "80px 40px 60px"
            align: "center"
            bg: "linear-gradient(180deg, #0a0e17 0%, #111827 100%)"
        }

        div {
            style {
                maxw: "800px"
                m: "0 auto"
            }

            # Logo
            img {
                src: "{logo}"
                style {
                    w: "140px"
                    h: "auto"
                    mb: "24px"
                    display: "block"
                    m: "0 auto 24px"
                }
            }

            # Badge
            div {
                style {
                    display: "inline-block"
                    bg: "rgba(249, 115, 22, 0.08)"
                    border: "1px solid rgba(249, 115, 22, 0.2)"
                    radius: "999px"
                    pad: "6px 18px"
                    mb: "24px"
                    size: "0.8rem"
                    fg: "#f97316"
                    letter-spacing: "0.05em"
                }
                text "{hero_badge}"
            }

            # Title
            h1 "{hero_title}" {
                style {
                    size: "4.5rem"
                    weight: "700"
                    mb: "16px"
                    lh: "1.05"
                    letter-spacing: "-0.03em"
                    fg: "#f9fafb"
                }
            }

            # Subtitle
            p "{hero_subtitle}" {
                style {
                    size: "1.3rem"
                    lh: "1.5"
                    fg: "#9ca3af"
                    maxw: "580px"
                    m: "0 auto 40px"
                    font-style: "italic"
                }
            }

            # CTA buttons
            div {
                style {
                    row
                    center
                    gap: "12px"
                    mb: "56px"
                }

                a "Get Started" {
                    style {
                        bg: "#f97316"
                        fg: "#0a0e17"
                        pad: "12px 28px"
                        radius: "8px"
                        size: "0.95rem"
                        weight: "600"
                        text-decoration: "none"
                        pointer
                        transition: "background 0.15s"
                    }
                    href: "#install"
                }

                a "View Source" {
                    style {
                        bg: "transparent"
                        fg: "#e5e7eb"
                        pad: "12px 28px"
                        radius: "8px"
                        size: "0.95rem"
                        weight: "500"
                        border: "1px solid #374151"
                        text-decoration: "none"
                        pointer
                        transition: "border-color 0.15s"
                    }
                    href: "https://github.com/hansdevs/rustscript"
                }
            }

            # Stats row
            div {
                style {
                    row
                    center
                    gap: "48px"
                }

                div {
                    style { align: "center" }
                    p "{hero_stat_1_value}" {
                        style { size: "2rem" weight: "700" fg: "#f9fafb" mb: "4px" }
                    }
                    p "{hero_stat_1_label}" {
                        style { size: "0.8rem" fg: "#6b7280" uppercase letter-spacing: "0.08em" }
                    }
                }

                div {
                    style { align: "center" }
                    p "{hero_stat_2_value}" {
                        style { size: "2rem" weight: "700" fg: "#f9fafb" mb: "4px" }
                    }
                    p "{hero_stat_2_label}" {
                        style { size: "0.8rem" fg: "#6b7280" uppercase letter-spacing: "0.08em" }
                    }
                }

                div {
                    style { align: "center" }
                    p "{hero_stat_3_value}" {
                        style { size: "2rem" weight: "700" fg: "#f9fafb" mb: "4px" }
                    }
                    p "{hero_stat_3_label}" {
                        style { size: "0.8rem" fg: "#6b7280" uppercase letter-spacing: "0.08em" }
                    }
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  ABOUT / ABSTRACT
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "80px 40px"
            bg: "#0a0e17"
        }

        div {
            style {
                maxw: "720px"
                m: "0 auto"
            }

            h2 "About" {
                style {
                    size: "1.8rem"
                    weight: "600"
                    mb: "32px"
                    align: "center"
                    fg: "#f9fafb"
                }
            }

            p "{about_p1}" {
                style {
                    size: "1rem"
                    lh: "1.8"
                    fg: "#9ca3af"
                    mb: "20px"
                }
            }

            p "{about_p2}" {
                style {
                    size: "1rem"
                    lh: "1.8"
                    fg: "#9ca3af"
                    mb: "20px"
                }
            }

            p "{about_p3}" {
                style {
                    size: "1rem"
                    lh: "1.8"
                    fg: "#9ca3af"
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  HOW IT WORKS
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "80px 40px"
            bg: "#111827"
        }

        div {
            style {
                maxw: "800px"
                m: "0 auto"
            }

            h2 "How it works" {
                style {
                    size: "1.8rem"
                    weight: "600"
                    mb: "48px"
                    align: "center"
                    fg: "#f9fafb"
                }
            }

            div {
                style {
                    row
                    center
                    gap: "32px"
                    flex-wrap: "wrap"
                    justify: "center"
                }

                # Step 1
                div {
                    style {
                        bg: "#0a0e17"
                        border: "1px solid #1f2937"
                        radius: "12px"
                        pad: "28px"
                        w: "200px"
                        align: "center"
                    }
                    p "{step_1_num}" {
                        style { size: "0.75rem" fg: "#f97316" weight: "600" mb: "12px" letter-spacing: "0.1em" }
                    }
                    p "{step_1_title}" {
                        style { size: "1.1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{step_1_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.5" }
                    }
                }

                # Arrow
                div {
                    style { fg: "#374151" size: "1.5rem" }
                    text "--->"
                }

                # Step 2
                div {
                    style {
                        bg: "#0a0e17"
                        border: "1px solid #1f2937"
                        radius: "12px"
                        pad: "28px"
                        w: "200px"
                        align: "center"
                    }
                    p "{step_2_num}" {
                        style { size: "0.75rem" fg: "#f97316" weight: "600" mb: "12px" letter-spacing: "0.1em" }
                    }
                    p "{step_2_title}" {
                        style { size: "1.1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{step_2_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.5" font-family: "'JetBrains Mono', monospace" }
                    }
                }

                # Arrow
                div {
                    style { fg: "#374151" size: "1.5rem" }
                    text "--->"
                }

                # Step 3
                div {
                    style {
                        bg: "#0a0e17"
                        border: "1px solid #1f2937"
                        radius: "12px"
                        pad: "28px"
                        w: "200px"
                        align: "center"
                    }
                    p "{step_3_num}" {
                        style { size: "0.75rem" fg: "#f97316" weight: "600" mb: "12px" letter-spacing: "0.1em" }
                    }
                    p "{step_3_title}" {
                        style { size: "1.1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{step_3_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.5" }
                    }
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  FEATURES
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "80px 40px"
            bg: "#0a0e17"
        }

        div {
            style {
                maxw: "900px"
                m: "0 auto"
            }

            h2 "What you get" {
                style {
                    size: "1.8rem"
                    weight: "600"
                    mb: "12px"
                    align: "center"
                    fg: "#f9fafb"
                }
            }

            p "Everything in the box. Nothing to install separately." {
                style {
                    align: "center"
                    fg: "#6b7280"
                    mb: "48px"
                    size: "1rem"
                }
            }

            div {
                style {
                    display: "grid"
                    grid-template-columns: "repeat(3, 1fr)"
                    gap: "16px"
                }

                # Feature 1
                div {
                    style {
                        bg: "#111827"
                        border: "1px solid #1f2937"
                        radius: "10px"
                        pad: "24px"
                    }
                    p "{f1_tag}" {
                        style {
                            font-family: "'JetBrains Mono', monospace"
                            size: "0.75rem"
                            fg: "#f97316"
                            bg: "rgba(249, 115, 22, 0.08)"
                            pad: "3px 8px"
                            radius: "4px"
                            display: "inline-block"
                            mb: "14px"
                        }
                    }
                    h3 "{f1_title}" {
                        style { size: "1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{f1_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.55" }
                    }
                }

                # Feature 2
                div {
                    style {
                        bg: "#111827"
                        border: "1px solid #1f2937"
                        radius: "10px"
                        pad: "24px"
                    }
                    p "{f2_tag}" {
                        style {
                            font-family: "'JetBrains Mono', monospace"
                            size: "0.75rem"
                            fg: "#f97316"
                            bg: "rgba(249, 115, 22, 0.08)"
                            pad: "3px 8px"
                            radius: "4px"
                            display: "inline-block"
                            mb: "14px"
                        }
                    }
                    h3 "{f2_title}" {
                        style { size: "1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{f2_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.55" }
                    }
                }

                # Feature 3
                div {
                    style {
                        bg: "#111827"
                        border: "1px solid #1f2937"
                        radius: "10px"
                        pad: "24px"
                    }
                    p "{f3_tag}" {
                        style {
                            font-family: "'JetBrains Mono', monospace"
                            size: "0.75rem"
                            fg: "#f97316"
                            bg: "rgba(249, 115, 22, 0.08)"
                            pad: "3px 8px"
                            radius: "4px"
                            display: "inline-block"
                            mb: "14px"
                        }
                    }
                    h3 "{f3_title}" {
                        style { size: "1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{f3_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.55" }
                    }
                }

                # Feature 4
                div {
                    style {
                        bg: "#111827"
                        border: "1px solid #1f2937"
                        radius: "10px"
                        pad: "24px"
                    }
                    p "{f4_tag}" {
                        style {
                            font-family: "'JetBrains Mono', monospace"
                            size: "0.75rem"
                            fg: "#f97316"
                            bg: "rgba(249, 115, 22, 0.08)"
                            pad: "3px 8px"
                            radius: "4px"
                            display: "inline-block"
                            mb: "14px"
                        }
                    }
                    h3 "{f4_title}" {
                        style { size: "1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{f4_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.55" }
                    }
                }

                # Feature 5
                div {
                    style {
                        bg: "#111827"
                        border: "1px solid #1f2937"
                        radius: "10px"
                        pad: "24px"
                    }
                    p "{f5_tag}" {
                        style {
                            font-family: "'JetBrains Mono', monospace"
                            size: "0.75rem"
                            fg: "#f97316"
                            bg: "rgba(249, 115, 22, 0.08)"
                            pad: "3px 8px"
                            radius: "4px"
                            display: "inline-block"
                            mb: "14px"
                        }
                    }
                    h3 "{f5_title}" {
                        style { size: "1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{f5_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.55" }
                    }
                }

                # Feature 6
                div {
                    style {
                        bg: "#111827"
                        border: "1px solid #1f2937"
                        radius: "10px"
                        pad: "24px"
                    }
                    p "{f6_tag}" {
                        style {
                            font-family: "'JetBrains Mono', monospace"
                            size: "0.75rem"
                            fg: "#f97316"
                            bg: "rgba(249, 115, 22, 0.08)"
                            pad: "3px 8px"
                            radius: "4px"
                            display: "inline-block"
                            mb: "14px"
                        }
                    }
                    h3 "{f6_title}" {
                        style { size: "1rem" weight: "600" fg: "#f9fafb" mb: "8px" }
                    }
                    p "{f6_desc}" {
                        style { size: "0.85rem" fg: "#6b7280" lh: "1.55" }
                    }
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  LIVE DEMO
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "80px 40px"
            bg: "#111827"
        }

        div {
            style {
                maxw: "700px"
                m: "0 auto"
            }

            h2 "Try it" {
                style {
                    size: "1.8rem"
                    weight: "600"
                    mb: "12px"
                    align: "center"
                    fg: "#f9fafb"
                }
            }

            p "This counter is running inside the compiled page right now." {
                style {
                    align: "center"
                    fg: "#6b7280"
                    mb: "40px"
                    size: "1rem"
                }
            }

            # Demo card
            div {
                style {
                    bg: "#0a0e17"
                    border: "1px solid #1f2937"
                    radius: "12px"
                    pad: "40px"
                    align: "center"
                }

                p "{demo_count}" {
                    style {
                        size: "4rem"
                        weight: "700"
                        fg: "#f97316"
                        mb: "8px"
                    }
                }

                p "{get_click_text(demo_count)}" {
                    style {
                        fg: "#6b7280"
                        mb: "28px"
                        size: "0.95rem"
                    }
                }

                div {
                    style {
                        row
                        center
                        gap: "12px"
                    }

                    button "- 1" {
                        style {
                            bg: "#1f2937"
                            fg: "#e5e7eb"
                            border: "1px solid #374151"
                            pad: "10px 24px"
                            radius: "8px"
                            size: "0.95rem"
                            pointer
                            transition: "background 0.15s"
                        }
                        on click {
                            demo_count = demo_count - 1
                        }
                    }

                    button "Reset" {
                        style {
                            bg: "#1f2937"
                            fg: "#9ca3af"
                            border: "1px solid #374151"
                            pad: "10px 20px"
                            radius: "8px"
                            size: "0.85rem"
                            pointer
                            transition: "background 0.15s"
                        }
                        on click {
                            demo_count = 0
                        }
                    }

                    button "+ 1" {
                        style {
                            bg: "#f97316"
                            fg: "#0a0e17"
                            border: "none"
                            pad: "10px 24px"
                            radius: "8px"
                            size: "0.95rem"
                            weight: "600"
                            pointer
                            transition: "background 0.15s"
                        }
                        on click {
                            demo_count = demo_count + 1
                        }
                    }
                }
            }

            # Source code preview
            div {
                style {
                    bg: "#0a0e17"
                    border: "1px solid #1f2937"
                    radius: "12px"
                    pad: "24px"
                    mt: "16px"
                    align: "left"
                }

                p "Source" {
                    style {
                        size: "0.7rem"
                        fg: "#6b7280"
                        uppercase
                        letter-spacing: "0.1em"
                        mb: "12px"
                    }
                }

                # Tab buttons
                div {
                    style {
                        row
                        gap: "8px"
                        mb: "16px"
                    }

                    button "Logic" {
                        style {
                            bg: "rgba(249, 115, 22, 0.1)"
                            fg: "#f97316"
                            border: "1px solid rgba(249, 115, 22, 0.2)"
                            pad: "5px 14px"
                            radius: "6px"
                            size: "0.8rem"
                            pointer
                        }
                        on click {
                            active_sample = "logic"
                        }
                    }

                    button "UI" {
                        style {
                            bg: "transparent"
                            fg: "#6b7280"
                            border: "1px solid #1f2937"
                            pad: "5px 14px"
                            radius: "6px"
                            size: "0.8rem"
                            pointer
                        }
                        on click {
                            active_sample = "ui"
                        }
                    }
                }

                if active_sample == "logic" {
                    p "{sample_logic}" {
                        style {
                            font-family: "'JetBrains Mono', monospace"
                            size: "0.82rem"
                            fg: "#9ca3af"
                            lh: "1.7"
                            white-space: "pre"
                        }
                    }
                }

                if active_sample == "ui" {
                    p "{sample_ui}" {
                        style {
                            font-family: "'JetBrains Mono', monospace"
                            size: "0.82rem"
                            fg: "#9ca3af"
                            lh: "1.7"
                            white-space: "pre"
                        }
                    }
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  INSTALL
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "80px 40px"
            bg: "#0a0e17"
        }

        div {
            style {
                maxw: "600px"
                m: "0 auto"
                align: "center"
            }

            h2 "Install" {
                style {
                    size: "1.8rem"
                    weight: "600"
                    mb: "12px"
                    fg: "#f9fafb"
                }
            }

            p "One command. No Rust toolchain required." {
                style {
                    fg: "#6b7280"
                    mb: "32px"
                    size: "1rem"
                }
            }

            div {
                style {
                    bg: "#111827"
                    border: "1px solid #1f2937"
                    radius: "10px"
                    pad: "20px 24px"
                    align: "left"
                    mb: "24px"
                }

                p "{install_cmd}" {
                    style {
                        font-family: "'JetBrains Mono', monospace"
                        size: "0.82rem"
                        fg: "#9ca3af"
                        lh: "1.5"
                        overflow: "auto"
                    }
                }
            }

            p "Or build from source:" {
                style { fg: "#6b7280" mb: "12px" size: "0.9rem" }
            }

            div {
                style {
                    bg: "#111827"
                    border: "1px solid #1f2937"
                    radius: "10px"
                    pad: "20px 24px"
                    align: "left"
                }

                p "git clone https://github.com/hansdevs/rustscript\ncd rustscript && make install" {
                    style {
                        font-family: "'JetBrains Mono', monospace"
                        size: "0.82rem"
                        fg: "#9ca3af"
                        lh: "1.7"
                        white-space: "pre"
                    }
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  FOOTER
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "40px"
            align: "center"
            border-top: "1px solid #1f2937"
        }

        p "{footer_text}" {
            style { fg: "#4b5563" size: "0.85rem" mb: "4px" }
        }
        p "{footer_sub}" {
            style { fg: "#374151" size: "0.75rem" }
        }
    }
}
