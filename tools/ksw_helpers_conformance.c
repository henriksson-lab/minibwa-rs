#include "../minibwa/ksw2.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static void print_cigar(const uint32_t *cigar, int n_cigar)
{
    int i;
    printf(" %d", n_cigar);
    for (i = 0; i < n_cigar; ++i) printf(" %u", cigar[i]);
}

static void run_mat_case(int id, int a, int b, int b_ts, int b_ambi, int mt)
{
    int i;
    int8_t mat[25];
    ksw_gen_nt4_mat(mat, (int8_t)a, (int8_t)b, (int8_t)b_ts, (int8_t)b_ambi, mt);
    printf("mat %d %d %d %d %d %d", id, a, b, b_ts, b_ambi, mt);
    for (i = 0; i < 25; ++i) printf(" %d", (int)mat[i]);
    putchar('\n');
}

static void run_zdrop_case(int id, uint32_t max, int max_t, int max_q, int is_rot,
                           int H, int a, int b, int zdrop, int e)
{
    ksw_extz_t ez = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
    int ret;
    ez.max = max;
    ez.max_t = max_t;
    ez.max_q = max_q;
    ret = ksw_apply_zdrop(&ez, is_rot, H, a, b, zdrop, (int8_t)e);
    printf("zdrop %d %u %d %d %d %d %d %d %d %d %d %u %u %d %d\n",
           id, max, max_t, max_q, is_rot, H, a, b, zdrop, e, ret, ez.max,
           ez.zdropped, ez.max_t, ez.max_q);
}

static void run_reset_case(int id)
{
    ksw_extz_t ez = {123, 1, 7, 8, 9, 10, 11, 12, 13, 14, 15, 1, 0};
    ksw_reset_extz(&ez);
    printf("reset %d %u %u %d %d %d %d %d %d %d %d %d\n", id, ez.max, ez.zdropped,
           ez.max_q, ez.max_t, ez.mqe, ez.mqe_t, ez.mte, ez.mte_q, ez.score,
           ez.n_cigar, ez.reach_end);
}

static void print_backtrack_case(int id, int is_rot, int is_rev, int min_intron_len,
                                 int n_col, int i0, int j0, int off_len,
                                 const int *off, const int *off_end, int p_len,
                                 const uint8_t *p)
{
    int i, m_cigar = 0, n_cigar = 0;
    uint32_t *cigar = 0;
    ksw_backtrack(NULL, is_rot, is_rev, min_intron_len, p, off, off_end, n_col,
                  i0, j0, &m_cigar, &n_cigar, &cigar);
    printf("bt %d %d %d %d %d %d %d %d %d", id, is_rot, is_rev, min_intron_len,
           n_col, i0, j0, off_len, p_len);
    for (i = 0; i < off_len; ++i) printf(" %d", off[i]);
    printf(" %d", off_end ? 1 : 0);
    if (off_end) {
        for (i = 0; i < off_len; ++i) printf(" %d", off_end[i]);
    }
    for (i = 0; i < p_len; ++i) printf(" %u", (unsigned)p[i]);
    printf(" %d", m_cigar);
    print_cigar(cigar, n_cigar);
    putchar('\n');
    free(cigar);
}

int main(void)
{
    int id = 0;
    run_reset_case(id++);

    run_mat_case(id++, 2, 4, 0, 1, 0);
    run_mat_case(id++, -3, -5, 2, -1, 0);
    run_mat_case(id++, 5, 1, 1, 0, 0);
    run_mat_case(id++, 7, 3, -2, 4, 0);
    run_mat_case(id++, 2, 4, 1, 3, 1);
    run_mat_case(id++, 2, 4, 1, 3, 2);
    run_mat_case(id++, -3, 5, -2, -1, 1);
    run_mat_case(id++, -128, 4, 1, 3, 0);

    run_zdrop_case(id++, 0, -1, -1, 0, 12, 3, 4, 5, 2);
    run_zdrop_case(id++, 12, 3, 4, 0, 1, 8, 8, 5, 2);
    run_zdrop_case(id++, 20, 5, 6, 1, 3, 13, 8, 7, 1);
    run_zdrop_case(id++, 20, 5, 6, 1, 18, 8, 7, -1, 3);
    run_zdrop_case(id++, 20, 5, 6, 0, 9, 7, 4, 100, 2);

    {
        static const int off[] = {0, 0, 0};
        static const int off_end[] = {2, 2, 2};
        static const uint8_t p_match[] = {0, 0, 0, 0, 0, 0, 0, 0, 0};
        static const uint8_t p_mix[] = {0, 2, 0, 1 | 8, 0, 2 | 16, 0, 1, 0};
        print_backtrack_case(id++, 0, 0, 0, 3, 2, 2, 3, off, off_end, 9, p_match);
        print_backtrack_case(id++, 0, 1, 0, 3, 2, 2, 3, off, off_end, 9, p_mix);
        print_backtrack_case(id++, 0, 0, 2, 3, 2, 2, 3, off, off_end, 9, p_mix);
    }
    {
        static const int off[] = {0, 0, 1, 1, 2};
        static const int off_end[] = {0, 1, 2, 3, 3};
        static const uint8_t p_rot[] = {
            0, 0, 0, 0,
            0, 1, 0, 0,
            2, 0, 1 | 8, 0,
            0, 2 | 16, 0, 0,
            0, 1, 0, 0,
        };
        print_backtrack_case(id++, 1, 0, 0, 4, 2, 2, 5, off, off_end, 20, p_rot);
        print_backtrack_case(id++, 1, 1, 3, 4, 2, 2, 5, off, off_end, 20, p_rot);
    }
    {
        static const int off[] = {1, 1, 1, 1};
        static const uint8_t p_force[] = {0, 0, 0, 0, 0, 0, 0, 0};
        print_backtrack_case(id++, 0, 0, 4, 2, 3, 0, 4, off, 0, 8, p_force);
    }

    return 0;
}
