#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    printf("========================================\n");
    printf("       OZ+ ETKILESIMLI KONSOL           \n");
    printf("========================================\n");
    printf("Cikmak icin 'cikis' yazin.\n");

    char history[10240] = "";
    char line[512];

    while (1) {
        printf(">>> ");
        if (!fgets(line, sizeof(line), stdin)) break;
        
        line[strcspn(line, "\n")] = 0;

        if (strcmp(line, "cikis") == 0) break;
        if (strlen(line) == 0) continue;

        int kaydet = 1;
        char temp_line[1024];
        strcpy(temp_line, line);

        if (strncmp(line, "yazdir ", 7) == 0) {
            kaydet = 0;
        } else if (strchr(line, '=') == NULL && strncmp(line, "degisken", 8) != 0 && strncmp(line, "sinif", 5) != 0 && strncmp(line, "islev", 5) != 0) {
            snprintf(temp_line, sizeof(temp_line), "yazdir (%s)", line);
            kaydet = 0;
        }

        FILE *f = fopen(".gecici_repl.ozp", "w");
        if (f) {
            fprintf(f, "%s\n%s\n", history, temp_line);
            fclose(f);
        }

        // Derleyicimizi cagiriyoruz (Python vs yok, tamamen saf C ve binary)
        int res = system("./oz .gecici_repl.ozp > .gecici_repl_out 2>&1");
        
        if (res == 0) {
            FILE *fout = fopen(".gecici_repl_out", "r");
            if (fout) {
                char out_line[512];
                while (fgets(out_line, sizeof(out_line), fout)) {
                    if (strncmp(out_line, "[1/3]", 5) == 0 || 
                        strncmp(out_line, "[2/3]", 5) == 0 || 
                        strncmp(out_line, "[3/3]", 5) == 0 || 
                        strncmp(out_line, "===", 3) == 0 ||
                        strncmp(out_line, "Basariyla", 9) == 0) {
                        continue;
                    }
                    // Eger ciktida sadece yeni satir veya bosluk varsa atla (ilk bastaki boslugu silmek icin)
                    if (out_line[0] != '\n') {
                        printf("%s", out_line);
                    }
                }
                fclose(fout);
            }
            if (kaydet) {
                strcat(history, line);
                strcat(history, "\n");
            }
        } else {
            printf("Hata: Gecersiz islem veya tanimsiz kelime.\n");
        }
        
        remove(".gecici_repl.ozp");
        remove(".gecici_repl_out");
    }

    printf("Gorusmek uzere!\n");
    return 0;
}
