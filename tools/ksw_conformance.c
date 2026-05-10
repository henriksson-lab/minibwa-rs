#include "ksw2.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static const int8_t ksw_conf_mat[25] = {
    2, -4, -4, -4, -1,
    -4, 2, -4, -4, -1,
    -4, -4, 2, -4, -1,
    -4, -4, -4, 2, -1,
    -1, -1, -1, -1, -1,
};

static const int8_t ksw_conf_alt_mat[25] = {
    3, -2, -2, -2, -2,
    -2, 3, -2, -2, -2,
    -2, -2, 3, -2, -2,
    -2, -2, -2, 3, -2,
    -2, -2, -2, -2, -2,
};

static const int8_t ksw_conf_peak_mat[25] = {
    7, -3, -3, -3, -2,
    -3, 7, -3, -3, -2,
    -3, -3, 7, -3, -2,
    -3, -3, -3, 7, -2,
    -2, -2, -2, -2, -2,
};

static uint32_t ksw_conf_rand(uint32_t *state)
{
    *state = *state * 1664525u + 1013904223u;
    return *state;
}

static void ksw_conf_fill(uint8_t *seq, int len, uint32_t *state)
{
    int i;
    for (i = 0; i < len; ++i) seq[i] = (uint8_t)(ksw_conf_rand(state) % 5u);
}

static void ksw_conf_print_seq(const uint8_t *seq, int len)
{
    int i;
    for (i = 0; i < len; ++i) putchar((int)('0' + seq[i]));
}

static void ksw_conf_print_result(const char *kind, int id, int qlen, const uint8_t *query,
                                  int tlen, const uint8_t *target, int q, int e, int q2,
                                  int e2, int w, int zdrop, int end_bonus, int flag, int mat_id,
                                  const ksw_extz_t *ez)
{
    int i;
    printf("%s %d %d %d %d %d %d %d %d %d %d %d %d ", kind, id, mat_id, qlen, tlen, q, e,
           q2, e2, w, zdrop, end_bonus, flag);
    ksw_conf_print_seq(query, qlen);
    putchar(' ');
    ksw_conf_print_seq(target, tlen);
    printf(" %u %u %d %d %d %d %d %d %d %d %d %d", ez->max, ez->zdropped, ez->max_q,
           ez->max_t, ez->mqe, ez->mqe_t, ez->mte, ez->mte_q, ez->score, ez->m_cigar,
           ez->n_cigar, ez->reach_end);
    for (i = 0; i < ez->n_cigar; ++i) printf(" %u", ez->cigar[i]);
    putchar('\n');
}

static void ksw_conf_run_case_mat(const char *kind, int id, int mat_id, int qlen,
                                  const uint8_t *query, int tlen, const uint8_t *target,
                                  int q, int e, int q2, int e2, int w, int zdrop,
                                  int end_bonus, int flag)
{
    ksw_extz_t ez = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
    const int8_t *mat = mat_id == 0 ? ksw_conf_mat : mat_id == 1 ? ksw_conf_alt_mat : ksw_conf_peak_mat;
    if (kind[0] == 'z') {
        ksw_extz2_sse(NULL, qlen, query, tlen, target, 5, mat, (int8_t)q, (int8_t)e,
                      w, zdrop, end_bonus, flag, &ez);
    } else {
        ksw_extd2_sse(NULL, qlen, query, tlen, target, 5, mat, (int8_t)q, (int8_t)e,
                      (int8_t)q2, (int8_t)e2, w, zdrop, end_bonus, flag, &ez);
    }
    ksw_conf_print_result(kind, id, qlen, query, tlen, target, q, e, q2, e2, w, zdrop,
                          end_bonus, flag, mat_id, &ez);
    free(ez.cigar);
}

static void ksw_conf_run_case(const char *kind, int id, int qlen, const uint8_t *query,
                              int tlen, const uint8_t *target, int q, int e, int q2, int e2,
                              int w, int zdrop, int end_bonus, int flag)
{
    ksw_conf_run_case_mat(kind, id, 0, qlen, query, tlen, target, q, e, q2, e2, w, zdrop,
                          end_bonus, flag);
}

int main(void)
{
    static const int flags[] = {
        0,
        KSW_EZ_SCORE_ONLY,
        KSW_EZ_RIGHT,
        KSW_EZ_EXTZ_ONLY,
        KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR,
        KSW_EZ_APPROX_MAX | KSW_EZ_SCORE_ONLY,
        KSW_EZ_APPROX_MAX | KSW_EZ_APPROX_DROP | KSW_EZ_SCORE_ONLY,
        KSW_EZ_GENERIC_SC | KSW_EZ_SCORE_ONLY,
        KSW_EZ_GENERIC_SC,
        KSW_EZ_GENERIC_SC | KSW_EZ_RIGHT,
        KSW_EZ_GENERIC_SC | KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR,
    };
    static const int bands[] = {-1, 0, 1, 2, 4, 8, 20};
    uint8_t query[128], target[128];
    uint32_t state = 0x5eed1234u;
    int id = 0, i, j;

    {
        static const uint8_t q0[] = {0, 1, 2, 3};
        static const uint8_t t0[] = {0, 1, 2, 3};
        static const uint8_t q1[] = {0, 1, 4, 3, 0, 2};
        static const uint8_t t1[] = {0, 4, 1, 3, 2};
        static const uint8_t q2s[] = {0, 1, 4, 3, 0, 2, 1, 1, 3, 4, 0, 2};
        static const uint8_t q3[] = {
            0, 1, 2, 3, 0, 1, 2, 3, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 4, 2,
            3, 0, 1, 2, 3, 0, 1, 2, 4, 4, 0, 1, 2, 3, 0, 1, 2, 3,
        };
        static const uint8_t t3[] = {
            0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 4, 4, 0, 1, 2, 3, 0, 1, 2,
            3, 0, 1, 2, 3, 0, 4, 1, 2, 3, 0, 1, 2, 3, 0,
        };
        ksw_conf_run_case("z", id++, 4, q0, 4, t0, 5, 1, 7, 1, -1, 100, 0, 0);
        ksw_conf_run_case("d", id++, 4, q0, 4, t0, 5, 1, 7, 1, -1, 100, 0, 0);
        ksw_conf_run_case("z", id++, 6, q1, 5, t1, 5, 1, 7, 1, 1, 3, 0,
                          KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR);
        ksw_conf_run_case("d", id++, 6, q1, 5, t1, 5, 1, 3, 1, 1, 3, 0,
                          KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR);
        ksw_conf_run_case("z", id++, 12, q2s, 1, t1, 5, 1, 7, 1, 0, 0, 0, 0);
        ksw_conf_run_case("d", id++, 12, q2s, 1, t1, 5, 1, 7, 1, 0, 0, 0, 0);
        ksw_conf_run_case("z", id++, 40, q3, 37, t3, 5, 1, 7, 1, 20, 12, 3,
                          KSW_EZ_RIGHT);
        ksw_conf_run_case("d", id++, 40, q3, 37, t3, 5, 1, 8, 2, 20, 12, 3,
                          KSW_EZ_RIGHT);
        ksw_conf_run_case("z", id++, 40, q3, 37, t3, 4, 2, 7, 1, -1, 30, 1,
                          KSW_EZ_GENERIC_SC);
        ksw_conf_run_case("d", id++, 40, q3, 37, t3, 4, 2, 9, 1, -1, 30, 1,
                          KSW_EZ_GENERIC_SC);
    }

    {
        static const int approx_flags[] = {
            KSW_EZ_APPROX_MAX,
            KSW_EZ_APPROX_MAX | KSW_EZ_APPROX_DROP,
            KSW_EZ_APPROX_MAX | KSW_EZ_RIGHT,
            KSW_EZ_APPROX_MAX | KSW_EZ_APPROX_DROP | KSW_EZ_RIGHT,
            KSW_EZ_APPROX_MAX | KSW_EZ_GENERIC_SC,
            KSW_EZ_APPROX_MAX | KSW_EZ_APPROX_DROP | KSW_EZ_GENERIC_SC,
            KSW_EZ_APPROX_MAX | KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR,
        };
        int n_flags = (int)(sizeof(approx_flags) / sizeof(approx_flags[0]));
        for (i = 0; i < 73; ++i) {
            query[i] = (uint8_t)((i * 7 + i / 5) % 5);
            target[i] = (uint8_t)((i * 11 + i / 7 + 2) % 5);
        }
        for (i = 0; i < n_flags; ++i) {
            int flag = approx_flags[i];
            int q = 4 + i % 4;
            int e = 1 + i % 3;
            int q2 = 3 + (i * 5) % 7;
            int e2 = 1 + (i * 3) % 4;
            int w = bands[(i * 2 + 1) % (sizeof(bands) / sizeof(bands[0]))];
            int zdrop = 6 + i * 3;
            int end_bonus = i % 5;
            ksw_conf_run_case_mat("z", id++, 0, 35, query, 32, target, q, e, q2, e2, w,
                                  zdrop, end_bonus, flag);
            ksw_conf_run_case_mat("d", id++, 0, 35, query, 32, target, q, e, q2, e2, w,
                                  zdrop, end_bonus, flag);
            ksw_conf_run_case_mat("z", id++, 1, 69, query, 73, target, q, e, q2, e2, w,
                                  zdrop, end_bonus, flag);
            ksw_conf_run_case_mat("d", id++, 1, 69, query, 73, target, q, e, q2, e2, w,
                                  zdrop, end_bonus, flag);
            ksw_conf_run_case_mat("z", id++, 0, 17, query + 3, 49, target + 5, q, e, q2, e2,
                                  w, zdrop / 2, end_bonus + 1, flag);
            ksw_conf_run_case_mat("d", id++, 0, 17, query + 3, 49, target + 5, q, e, q2, e2,
                                  w, zdrop / 2, end_bonus + 1, flag);
            ksw_conf_run_case_mat("z", id++, 1, 48, query + 2, 33, target + 4, q, e, q2, e2,
                                  w, zdrop / 2, end_bonus + 2, flag);
            ksw_conf_run_case_mat("d", id++, 1, 48, query + 2, 33, target + 4, q, e, q2, e2,
                                  w, zdrop / 2, end_bonus + 2, flag);
        }
    }

    {
        static const int stress_flags[] = {
            0,
            KSW_EZ_RIGHT,
            KSW_EZ_APPROX_MAX,
            KSW_EZ_APPROX_MAX | KSW_EZ_APPROX_DROP,
            KSW_EZ_GENERIC_SC,
            KSW_EZ_GENERIC_SC | KSW_EZ_RIGHT,
            KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR,
            KSW_EZ_APPROX_MAX | KSW_EZ_EXTZ_ONLY | KSW_EZ_RIGHT | KSW_EZ_REV_CIGAR,
        };
        int n_flags = (int)(sizeof(stress_flags) / sizeof(stress_flags[0]));
        for (i = 0; i < 127; ++i) {
            query[i] = (uint8_t)((i / 3 + i / 11) % 4);
            target[i] = (uint8_t)((i / 3 + (i + 5) / 17) % 4);
            if (i % 19 == 0) query[i] = 4;
            if (i % 23 == 0) target[i] = 4;
        }
        for (i = 0; i < n_flags; ++i) {
            int flag = stress_flags[i];
            int q = 5 + i % 3;
            int e = 1 + i % 2;
            int q2 = 4 + (i * 3) % 5;
            int e2 = 1 + (i * 5) % 3;
            int w = bands[(i * 4 + 2) % (sizeof(bands) / sizeof(bands[0]))];
            int zdrop = 8 + i * 4;
            int end_bonus = (i * 2) % 7;
            ksw_conf_run_case_mat("z", id++, 2, 16, query, 16, target, q, e, q2, e2, w,
                                  zdrop, end_bonus, flag);
            ksw_conf_run_case_mat("d", id++, 2, 16, query, 16, target, q, e, q2, e2, w,
                                  zdrop, end_bonus, flag);
            ksw_conf_run_case_mat("z", id++, 2, 31, query + 1, 32, target + 2, q, e, q2,
                                  e2, w, zdrop, end_bonus, flag);
            ksw_conf_run_case_mat("d", id++, 2, 31, query + 1, 32, target + 2, q, e, q2,
                                  e2, w, zdrop, end_bonus, flag);
            ksw_conf_run_case_mat("z", id++, 2, 63, query + 3, 65, target + 4, q, e, q2,
                                  e2, w, zdrop / 2, end_bonus + 1, flag);
            ksw_conf_run_case_mat("d", id++, 2, 63, query + 3, 65, target + 4, q, e, q2,
                                  e2, w, zdrop / 2, end_bonus + 1, flag);
            ksw_conf_run_case_mat("z", id++, 2, 96, query, 95, target + 1, q, e, q2, e2,
                                  w, zdrop / 2, end_bonus + 2, flag);
            ksw_conf_run_case_mat("d", id++, 2, 96, query, 95, target + 1, q, e, q2, e2,
                                  w, zdrop / 2, end_bonus + 2, flag);
        }
    }

    {
        static const int edge_lens[] = {
            15, 16, 17, 31, 32, 33, 47, 48, 49, 63, 64, 65, 95, 96, 97, 111, 112, 127,
        };
        int n_edge = (int)(sizeof(edge_lens) / sizeof(edge_lens[0]));
        for (i = 0; i < n_edge; ++i) {
            int qlen = edge_lens[i];
            int tlen = edge_lens[(i * 5 + 3) % n_edge];
            int q = 3 + i % 6;
            int e = 1 + i % 3;
            int q2 = 2 + (i * 7) % 8;
            int e2 = 1 + (i * 5) % 4;
            int w = bands[(i * 3) % (sizeof(bands) / sizeof(bands[0]))];
            int zdrop = (i % 5 == 0) ? 25 : (i % 7) - 1;
            int end_bonus = i % 9;
            int flag = flags[(i * 5) % (sizeof(flags) / sizeof(flags[0]))];
            ksw_conf_fill(query, qlen, &state);
            ksw_conf_fill(target, tlen, &state);
            ksw_conf_run_case("z", id++, qlen, query, tlen, target, q, e, q2, e2, w, zdrop,
                              end_bonus, flag);
            ksw_conf_run_case("d", id++, qlen, query, tlen, target, q, e, q2, e2, w, zdrop,
                              end_bonus, flag);
        }
        for (i = 0; i < n_edge; i += 3) {
            for (j = 0; j < n_edge; j += 5) {
                int qlen = edge_lens[i];
                int tlen = edge_lens[j];
                int q = 4 + (i + j) % 5;
                int e = 1 + (i + 2 * j) % 3;
                int q2 = 3 + (2 * i + j) % 7;
                int e2 = 1 + (3 * i + j) % 4;
                int w = bands[(i + j) % (sizeof(bands) / sizeof(bands[0]))];
                int zdrop = (i + j) % 37 - 1;
                int end_bonus = (i + 2 * j) % 11;
                int flag = flags[(i + 2 * j) % (sizeof(flags) / sizeof(flags[0]))];
                ksw_conf_fill(query, qlen, &state);
                ksw_conf_fill(target, tlen, &state);
                ksw_conf_run_case("z", id++, qlen, query, tlen, target, q, e, q2, e2, w,
                                  zdrop, end_bonus, flag);
                ksw_conf_run_case("d", id++, qlen, query, tlen, target, q, e, q2, e2, w,
                                  zdrop, end_bonus, flag);
            }
        }
    }

    for (i = 0; i < 360; ++i) {
        int qlen = 1 + (int)(ksw_conf_rand(&state) % 28u);
        int tlen = 1 + (int)(ksw_conf_rand(&state) % 28u);
        int q = 3 + (int)(ksw_conf_rand(&state) % 7u);
        int e = 1 + (int)(ksw_conf_rand(&state) % 3u);
        int q2 = 2 + (int)(ksw_conf_rand(&state) % 8u);
        int e2 = 1 + (int)(ksw_conf_rand(&state) % 4u);
        int w = bands[ksw_conf_rand(&state) % (sizeof(bands) / sizeof(bands[0]))];
        int zdrop = (int)(ksw_conf_rand(&state) % 25u) - 1;
        int end_bonus = (int)(ksw_conf_rand(&state) % 7u);
        int flag = flags[ksw_conf_rand(&state) % (sizeof(flags) / sizeof(flags[0]))];
        ksw_conf_fill(query, qlen, &state);
        ksw_conf_fill(target, tlen, &state);
        ksw_conf_run_case("z", id++, qlen, query, tlen, target, q, e, q2, e2, w, zdrop,
                          end_bonus, flag);
        ksw_conf_run_case("d", id++, qlen, query, tlen, target, q, e, q2, e2, w, zdrop,
                          end_bonus, flag);
    }
    for (i = 0; i < 60; ++i) {
        int qlen = 1 + (int)(ksw_conf_rand(&state) % 64u);
        int tlen = 1 + (int)(ksw_conf_rand(&state) % 64u);
        int q = 3 + (int)(ksw_conf_rand(&state) % 6u);
        int e = 1 + (int)(ksw_conf_rand(&state) % 3u);
        int q2 = 2 + (int)(ksw_conf_rand(&state) % 7u);
        int e2 = 1 + (int)(ksw_conf_rand(&state) % 4u);
        int w = bands[ksw_conf_rand(&state) % (sizeof(bands) / sizeof(bands[0]))];
        int zdrop = (int)(ksw_conf_rand(&state) % 31u) - 1;
        int end_bonus = (int)(ksw_conf_rand(&state) % 9u);
        int flag = flags[ksw_conf_rand(&state) % (sizeof(flags) / sizeof(flags[0]))];
        ksw_conf_fill(query, qlen, &state);
        ksw_conf_fill(target, tlen, &state);
        ksw_conf_run_case_mat("z", id++, 1, qlen, query, tlen, target, q, e, q2, e2, w,
                              zdrop, end_bonus, flag);
        ksw_conf_run_case_mat("d", id++, 1, qlen, query, tlen, target, q, e, q2, e2, w,
                              zdrop, end_bonus, flag);
    }
    for (i = 0; i < 240; ++i) {
        int qlen = 29 + (int)(ksw_conf_rand(&state) % 84u);
        int tlen = 29 + (int)(ksw_conf_rand(&state) % 84u);
        int q = 3 + (int)(ksw_conf_rand(&state) % 6u);
        int e = 1 + (int)(ksw_conf_rand(&state) % 3u);
        int q2 = 2 + (int)(ksw_conf_rand(&state) % 8u);
        int e2 = 1 + (int)(ksw_conf_rand(&state) % 4u);
        int w = bands[ksw_conf_rand(&state) % (sizeof(bands) / sizeof(bands[0]))];
        int zdrop = (int)(ksw_conf_rand(&state) % 40u) - 1;
        int end_bonus = (int)(ksw_conf_rand(&state) % 9u);
        int flag = flags[ksw_conf_rand(&state) % (sizeof(flags) / sizeof(flags[0]))];
        ksw_conf_fill(query, qlen, &state);
        ksw_conf_fill(target, tlen, &state);
        ksw_conf_run_case("z", id++, qlen, query, tlen, target, q, e, q2, e2, w, zdrop,
                          end_bonus, flag);
        ksw_conf_run_case("d", id++, qlen, query, tlen, target, q, e, q2, e2, w, zdrop,
                          end_bonus, flag);
    }
    return 0;
}
