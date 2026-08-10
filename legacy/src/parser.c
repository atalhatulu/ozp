#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "parser.h"

// Basit sembol tablosu ve tip yönetimi
static char* symbol_table[1024];
static int symbol_types[1024]; // 0: long, 1: metin(char*), 2: ondalik(double), 3: sozluk
static int symbol_scopes[1024];
static int symbol_count = 0;
static int current_scope = 0;

static char current_class_name[256] = "";
static int in_sinif_block = 0;

static int is_declared(const char* name) {
    for (int i = 0; i < symbol_count; i++) {
        if (strcmp(symbol_table[i], name) == 0) return 1;
    }
    return 0;
}

static int get_var_type(const char* name) {
    for (int i = 0; i < symbol_count; i++) {
        if (strcmp(symbol_table[i], name) == 0) return symbol_types[i];
    }
    return 0;
}

static void declare_var(const char* name, int type) {
    if (!is_declared(name)) {
        symbol_table[symbol_count] = strdup(name);
        symbol_types[symbol_count] = type;
        symbol_scopes[symbol_count] = current_scope;
        symbol_count++;
    }
}

static void oz_hata_ver(int satir, const char* mesaj) {
    printf("\n[OZ+ DERLEME HATASI] Satir %d: %s\n", satir, mesaj);
}

static void free_symbol_table() {
    for (int i = 0; i < symbol_count; i++) {
        free(symbol_table[i]);
    }
    symbol_count = 0;
}

int parse_and_transpile(TokenArray* tokens, const char* c_cikti_yolu) {
    FILE* out = fopen(c_cikti_yolu, "w");
    if (!out) {
        printf("Hata: Cikti dosyasi olusturulamadi!\n");
        return 0;
    }
    free_symbol_table(); // Eski sembolleri temizle

    fprintf(out, "#include <stdio.h>\n");
    fprintf(out, "#include \"matematik.h\"\n");
    fprintf(out, "#include \"oz_standart.h\"\n\n");
    fprintf(out, "int main() {\n");

    int is_global = 1;
    int in_eger = 0;
    int in_islev_decl = 0;

    for (int i = 0; i < tokens->count; i++) {
        Token t = tokens->tokens[i];
        
        if (i == 0 || tokens->tokens[i-1].satir < t.satir) {
            int indent = current_scope;
            if (t.tip == TOKEN_ANAHTAR && (strcmp(t.deger, "son") == 0 || strcmp(t.deger, "degilse") == 0 || strcmp(t.deger, "hata_yakala") == 0)) {
                indent = current_scope - 1;
                if (indent < 0) indent = 0;
            }
            for (int ind = 0; ind < indent; ind++) fprintf(out, "    ");
        }

        if (t.tip == TOKEN_EOF) break;

        if (t.tip == TOKEN_ANAHTAR) {
            if (strcmp(t.deger, "degisken") == 0) {
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID) declare_var(tokens->tokens[i+1].deger, 0);
                fprintf(out, "long ");
            } else if (strcmp(t.deger, "metin") == 0) {
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID) declare_var(tokens->tokens[i+1].deger, 1);
                fprintf(out, "char* ");
            } else if (strcmp(t.deger, "ondalik") == 0) {
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID) declare_var(tokens->tokens[i+1].deger, 2);
                fprintf(out, "double ");
            } else if (strcmp(t.deger, "dizi") == 0) {
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID) declare_var(tokens->tokens[i+1].deger, 0); // Varsayilan long array
                fprintf(out, "long ");
            } else if (strcmp(t.deger, "sozluk") == 0) {
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID) declare_var(tokens->tokens[i+1].deger, 3); // 3 = Sozluk
                fprintf(out, "OzSozluk* ");
            } else if (strcmp(t.deger, "dene") == 0) {
                fprintf(out, "if (setjmp(_oz_hata_buf) == 0) ");
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_IKINOKTA) {
                    fprintf(out, "{\n");
                    current_scope++;
                    i++;
                }
            } else if (strcmp(t.deger, "hata_yakala") == 0) {
                current_scope--;
                for(int j=symbol_count-1; j>=0; j--) {
                    if(symbol_scopes[j] > current_scope) { 
                        if (symbol_types[j] == 3) fprintf(out, "sozluk_sil(%s);\n", symbol_table[j]);
                        else if (symbol_types[j] == 4) fprintf(out, "free(%s);\n", symbol_table[j]);
                        free(symbol_table[j]); symbol_count--; 
                    }
                }
                fprintf(out, "} else ");
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_IKINOKTA) {
                    fprintf(out, "{\n");
                    current_scope++;
                    i++;
                }
            } else if (strcmp(t.deger, "sinif") == 0) {
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID) {
                    snprintf(current_class_name, sizeof(current_class_name), "%s", tokens->tokens[i+1].deger);
                    fprintf(out, "typedef struct ");
                    in_sinif_block = 1;
                    i++; // sinif adini atla
                }
            } else if (strcmp(t.deger, "yeni") == 0) {
                if (i + 2 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID && tokens->tokens[i+2].tip == TOKEN_ID) {
                    fprintf(out, "%s* %s = (%s*)malloc(sizeof(%s));\n", 
                            tokens->tokens[i+1].deger, tokens->tokens[i+2].deger, 
                            tokens->tokens[i+1].deger, tokens->tokens[i+1].deger);
                    declare_var(tokens->tokens[i+2].deger, 4); // 4 = Object
                    i += 2;
                }
            } else if (strcmp(t.deger, "hata_firlat") == 0) {
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_METIN) {
                    fprintf(out, "_oz_son_hata_mesaji = \"%s\"; longjmp(_oz_hata_buf, 1);\n", tokens->tokens[i+1].deger);
                    i++;
                } else {
                    fprintf(out, "_oz_son_hata_mesaji = \"Bilinmeyen Hata\"; longjmp(_oz_hata_buf, 1);\n");
                }
            } else if (strcmp(t.deger, "girdi") == 0) {
                if (i + 2 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID && 
                    strcmp(tokens->tokens[i+2].deger, "al") == 0) {
                    fprintf(out, "char %s[256] = \"\"; scanf(\" %%255[^\\n]\", %s); ", tokens->tokens[i+1].deger, tokens->tokens[i+1].deger);
                    declare_var(tokens->tokens[i+1].deger, 1); // 1 = metin
                    i += 2;
                } else if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ID) {
                    fprintf(out, "scanf(\"%%ld\", &%s);\n", tokens->tokens[i+1].deger);
                    i++;
                }
            } else if (strcmp(t.deger, "dongu") == 0) {
                fprintf(out, "while (");
            } else if (strcmp(t.deger, "ise") == 0) {
                fprintf(out, "if (");
            } else if (strcmp(t.deger, "degilse") == 0) {
                if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ANAHTAR && strcmp(tokens->tokens[i+1].deger, "ise") == 0) {
                    current_scope--;
                    for(int j=symbol_count-1; j>=0; j--) {
                        if(symbol_scopes[j] > current_scope) { 
                            if (symbol_types[j] == 3) fprintf(out, "sozluk_sil(%s);\n", symbol_table[j]);
                            else if (symbol_types[j] == 4) fprintf(out, "free(%s);\n", symbol_table[j]);
                            free(symbol_table[j]); symbol_count--; 
                        }
                    }
                    fprintf(out, "} else if (");
                    i++; // ise'yi atla
                } else {
                    current_scope--;
                    for(int j=symbol_count-1; j>=0; j--) {
                        if(symbol_scopes[j] > current_scope) { 
                            if (symbol_types[j] == 3) fprintf(out, "sozluk_sil(%s);\n", symbol_table[j]);
                            else if (symbol_types[j] == 4) fprintf(out, "free(%s);\n", symbol_table[j]);
                            free(symbol_table[j]); symbol_count--; 
                        }
                    }
                    fprintf(out, "} else ");
                    if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_IKINOKTA) {
                        fprintf(out, "{\n");
                        current_scope++;
                        i++; // IKINOKTA'yi atla
                    }
                }
            } else if (strcmp(t.deger, "son") == 0) {
                if (in_sinif_block) {
                    fprintf(out, "} %s;\n", current_class_name);
                    in_sinif_block = 0;
                    current_scope--;
                } else {
                    current_scope--;
                    for(int j=symbol_count-1; j>=0; j--) {
                        if(symbol_scopes[j] > current_scope) { 
                            if (symbol_types[j] == 3) {
                                fprintf(out, "sozluk_sil(%s);\n", symbol_table[j]);
                            } else if (symbol_types[j] == 4) {
                                fprintf(out, "free(%s);\n", symbol_table[j]);
                            }
                            free(symbol_table[j]); 
                            symbol_count--; 
                        }
                    }
                    fprintf(out, "}\n");
                }
            } else if (strcmp(t.deger, "yazdir") == 0) {
                if (i + 1 < tokens->count) {
                    if (tokens->tokens[i+1].tip == TOKEN_ID) {
                        if (strncmp(tokens->tokens[i+1].deger, "_oz_", 4) != 0 && !is_declared(tokens->tokens[i+1].deger)) {
                            oz_hata_ver(t.satir, "Kullanilmaya calisilan degisken daha once tanimlanmamis!");
                            fclose(out); return 0;
                        }
                        int is_array = 0;
                        char array_idx[64] = "";
                        if (i + 2 < tokens->count && tokens->tokens[i+2].tip == TOKEN_KOSELI_AC) {
                            is_array = 1;
                            snprintf(array_idx, sizeof(array_idx), "[%s]", tokens->tokens[i+3].deger); // Basitce icindeki id/sayiyi alalim
                            i += 3; // Koseyi kapatmak icin i'yi arttiracagiz
                            if (i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_KOSELI_KAPA) i++;
                        }
                        
                        int vtype = get_var_type(tokens->tokens[i+1 - (is_array ? 4 : 0)].deger);
                        if (strcmp(tokens->tokens[i+1 - (is_array ? 4 : 0)].deger, "_oz_son_hata_mesaji") == 0) vtype = 1;

                        if (vtype == 1) {
                            fprintf(out, "printf(\"%%s\\n\", %s%s);\n", tokens->tokens[i+1 - (is_array ? 4 : 0)].deger, array_idx);
                        } else if (vtype == 2) {
                            fprintf(out, "printf(\"%%f\\n\", %s%s);\n", tokens->tokens[i+1 - (is_array ? 4 : 0)].deger, array_idx);
                        } else {
                            fprintf(out, "printf(\"%%ld\\n\", %s%s);\n", tokens->tokens[i+1 - (is_array ? 4 : 0)].deger, array_idx);
                        }
                        i++;
                    } else if (tokens->tokens[i+1].tip == TOKEN_METIN) {
                        fprintf(out, "printf(\"%%s\\n\", \"%s\");\n", tokens->tokens[i+1].deger);
                        i++;
                    } else if (tokens->tokens[i+1].tip == TOKEN_SAYI) {
                        fprintf(out, "printf(\"%%ld\\n\", %sL);\n", tokens->tokens[i+1].deger);
                        i++;
                    } else if (tokens->tokens[i+1].tip == TOKEN_PARANTEZ_AC) {
                        // islem yazdirmaya calisiyor (5 + 3) gibi. Sadece int (long) desteklesin simdilik.
                        fprintf(out, "printf(\"%%ld\\n\", (long)");
                        i++;
                        while(i < tokens->count && tokens->tokens[i].satir == t.satir) {
                            // Satır sonuna kadar al
                            Token exp = tokens->tokens[i];
                            if(exp.tip == TOKEN_ID) fprintf(out, "%s ", exp.deger);
                            else if(exp.tip == TOKEN_SAYI) fprintf(out, "%s ", exp.deger);
                            else if(exp.tip == TOKEN_ARTI) fprintf(out, "+ ");
                            else if(exp.tip == TOKEN_EKS) fprintf(out, "- ");
                            else if(exp.tip == TOKEN_CARP) fprintf(out, "* ");
                            else if(exp.tip == TOKEN_BOL) fprintf(out, "/ ");
                            else if(exp.tip == TOKEN_PARANTEZ_AC) fprintf(out, "( ");
                            else if(exp.tip == TOKEN_PARANTEZ_KAPA) fprintf(out, ") ");
                            i++;
                        }
                        fprintf(out, ");\n");
                        i--; // Dıştaki for döngüsü bir artıracak, geri alalım
                    } else {
                        oz_hata_ver(t.satir, "'yazdir' komutundan sonra gecerli bir deger gelmeli!");
                        fclose(out); return 0;
                    }
                }
            } else if (strcmp(t.deger, "zaman_baslat") == 0) {
                fprintf(out, "_oz_zaman_baslat();\n");
            } else if (strcmp(t.deger, "zaman_bitir") == 0) {
                fprintf(out, "_oz_zaman_bitir();\n");
            } else if (strcmp(t.deger, "islev") == 0) {
                // islev isim: gibi varsayalım şimdilik
                fprintf(out, "long ");
                is_global = 0; // Fonksiyon başladığı için globali geçebiliriz
                in_islev_decl = 1;
            } else if (strcmp(t.deger, "don") == 0) {
                fprintf(out, "return ");
            } else if (strcmp(t.deger, "kir") == 0) {
                fprintf(out, "break;\n");
            } else if (strcmp(t.deger, "devam_et") == 0) {
                fprintf(out, "continue;\n");
            } else if (strcmp(t.deger, "sabit") == 0) {
                fprintf(out, "const ");
            } else if (strcmp(t.deger, "Dogru") == 0) {
                fprintf(out, "1 ");
            } else if (strcmp(t.deger, "Yanlis") == 0) {
                fprintf(out, "0 ");
            } else if (strcmp(t.deger, "hic") == 0) {
                fprintf(out, "NULL ");
            } else if (strcmp(t.deger, "eger") == 0) {
                in_eger = 1;
                fprintf(out, "( ");
            }
        } else if (t.tip == TOKEN_ID) {
            // Implicit declaration (Type Inference - Python like)
            if (strncmp(t.deger, "_oz_", 4) != 0 && !is_declared(t.deger) && i + 1 < tokens->count && tokens->tokens[i+1].tip == TOKEN_ESIT) {
                if (i + 2 < tokens->count && tokens->tokens[i+2].tip == TOKEN_METIN) {
                    fprintf(out, "char* ");
                    declare_var(t.deger, 1);
                } else if (i + 2 < tokens->count && tokens->tokens[i+2].tip == TOKEN_SAYI && strchr(tokens->tokens[i+2].deger, '.') != NULL) {
                    fprintf(out, "double ");
                    declare_var(t.deger, 2);
                } else if (i + 2 < tokens->count && tokens->tokens[i+2].tip == TOKEN_ID && strcmp(tokens->tokens[i+2].deger, "sozluk_olustur") == 0) {
                    fprintf(out, "Sozluk* ");
                    declare_var(t.deger, 3);
                } else if (i + 2 < tokens->count && tokens->tokens[i+2].tip == TOKEN_ANAHTAR && strcmp(tokens->tokens[i+2].deger, "hic") == 0) {
                    fprintf(out, "void* ");
                    declare_var(t.deger, 0); // void* icin type 0 veya ozel type
                } else {
                    fprintf(out, "long ");
                    declare_var(t.deger, 0);
                }
            }
            fprintf(out, "%s ", t.deger);
        } else if (t.tip == TOKEN_METIN) {
            fprintf(out, "\"%s\" ", t.deger);
        } else if (t.tip == TOKEN_SAYI) {
            fprintf(out, "%s ", t.deger);
        } else if (t.tip == TOKEN_ESIT) {
            fprintf(out, "= ");
        } else if (t.tip == TOKEN_ARTI) {
            fprintf(out, "+ ");
        } else if (t.tip == TOKEN_EKS) {
            fprintf(out, "- ");
        } else if (t.tip == TOKEN_CARP) {
            fprintf(out, "* ");
        } else if (t.tip == TOKEN_BOL) {
            fprintf(out, "/ ");
        } else if (t.tip == TOKEN_MOD) {
            fprintf(out, "%% ");
        } else if (t.tip == TOKEN_IKINOKTA) {
            if (in_eger) {
                fprintf(out, ") ? ");
            } else {
                current_scope++;
                if (in_islev_decl || in_sinif_block) {
                    fprintf(out, " {\n");
                    in_islev_decl = 0;
                } else {
                    fprintf(out, ") {\n");
                }
            }
        } else if (t.tip == TOKEN_ESIT_ESIT) {
            fprintf(out, "== ");
        } else if (t.tip == TOKEN_ESIT_DEGIL) {
            fprintf(out, "!= ");
        } else if (t.tip == TOKEN_VE) {
            fprintf(out, "&& ");
        } else if (t.tip == TOKEN_VEYA) {
            fprintf(out, "|| ");
        } else if (t.tip == TOKEN_PARANTEZ_AC) {
            fprintf(out, "( ");
        } else if (t.tip == TOKEN_PARANTEZ_KAPA) {
            fprintf(out, ") ");
        } else if (t.tip == TOKEN_KOSELI_AC) {
            fprintf(out, "[ ");
        } else if (t.tip == TOKEN_KOSELI_KAPA) {
            fprintf(out, "] ");
        } else if (t.tip == TOKEN_NOKTA) {
            fprintf(out, "->");
        } else if (t.tip == TOKEN_VIRGUL) {
            if (in_eger) {
                fprintf(out, " : ");
                in_eger = 0;
            } else {
                fprintf(out, ", ");
            }
        } else if (t.tip == TOKEN_KUCUK_ESIT) {
            fprintf(out, "<= ");
        } else if (t.tip == TOKEN_BUYUK_ESIT) {
            fprintf(out, ">= ");
        } else if (t.tip == TOKEN_KUCUK) {
            fprintf(out, "< ");
        } else if (t.tip == TOKEN_BUYUK) {
            fprintf(out, "> ");
        }

        // Noktalı virgül mantığı
        if (i + 1 < tokens->count) {
            Token next = tokens->tokens[i+1];
            if (next.satir > t.satir) {
                int needs_semi = 1;
                if (t.tip == TOKEN_IKINOKTA) needs_semi = 0;
                if (t.tip == TOKEN_ANAHTAR) {
                    if (strcmp(t.deger, "Dogru") == 0 || strcmp(t.deger, "Yanlis") == 0 ||
                        strcmp(t.deger, "hic") == 0 || strcmp(t.deger, "kir") == 0 ||
                        strcmp(t.deger, "devam_et") == 0 || strcmp(t.deger, "don") == 0) {
                        needs_semi = 1;
                    } else if (strcmp(t.deger, "degilse") == 0) {
                        fprintf(out, "{\n");
                        needs_semi = 0;
                    } else {
                        needs_semi = 0;
                    }
                }
                if (needs_semi) {
                    fprintf(out, ";\n");
                }
            }
        }
    }

    fprintf(out, "return 0;\n}\n");
    fclose(out);
    return 1;
}
