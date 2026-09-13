// Exemplo 1 do vídeo: programa sem erro léxico
public class MediaAluno {
    public static void main(String[] args) {
        int nota1 = 8;
        int nota2 = 7;
        float media = 7.5;
        char conceito = 'A';
        String nome = "Ana\n";

        /* calcula se o aluno passou */
        if (media >= 7.0 && nota1 != 0) {
            media += 0.5;
            return;
        } else {
            while (nota2 < 10) {
                nota2 = nota2 + 1;
            }
        }
    }
}
