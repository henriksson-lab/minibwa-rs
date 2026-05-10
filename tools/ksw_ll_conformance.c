#include "ksw2.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static const int8_t ksw_ll_conf_mat[25] = {
    2, -4, -4, -4, -1,
    -4, 2, -4, -4, -1,
    -4, -4, 2, -4, -1,
    -4, -4, -4, 2, -1,
    -1, -1, -1, -1, -1,
};

static const int8_t ksw_ll_conf_alt_mat[25] = {
    3, -2, -2, -2, -2,
    -2, 3, -2, -2, -2,
    -2, -2, 3, -2, -2,
    -2, -2, -2, 3, -2,
    -2, -2, -2, -2, -2,
};

static const int8_t ksw_ll_conf_peak_mat[25] = {
    7, -3, -3, -3, -2,
    -3, 7, -3, -3, -2,
    -3, -3, 7, -3, -2,
    -3, -3, -3, 7, -2,
    -2, -2, -2, -2, -2,
};

static uint32_t ksw_ll_conf_rand(uint32_t *state)
{
    *state = *state * 1664525u + 1013904223u;
    return *state;
}

static void ksw_ll_conf_fill(uint8_t *seq, int len, uint32_t *state)
{
    int i;
    for (i = 0; i < len; ++i) seq[i] = (uint8_t)(ksw_ll_conf_rand(state) % 5u);
}

static void ksw_ll_conf_print_seq(const uint8_t *seq, int len)
{
    int i;
    for (i = 0; i < len; ++i) putchar((int)('0' + seq[i]));
}

static void ksw_ll_conf_run_case_mat(int id, int mat_id, int size, int qlen,
                                     const uint8_t *query, int tlen, const uint8_t *target,
                                     int gapo, int gape, int xtra)
{
    const int8_t *mat = mat_id == 0 ? ksw_ll_conf_mat
        : mat_id == 1 ? ksw_ll_conf_alt_mat : ksw_ll_conf_peak_mat;
    void *q = ksw_ll_qinit(NULL, size, qlen, query, 5, mat);
    ksw_llrst_t r = size == 1
        ? ksw_ll_u8_core(q, tlen, target, gapo, gape, xtra)
        : ksw_ll_i16_core(q, tlen, target, gapo, gape, xtra);
    printf("%d %d %d %d %d %d %d %d ", id, mat_id, size, qlen, tlen, gapo, gape, xtra);
    ksw_ll_conf_print_seq(query, qlen);
    putchar(' ');
    ksw_ll_conf_print_seq(target, tlen);
    printf(" %d %d %d %d %d\n", r.score, r.te, r.qe, r.score2, r.te2);
    free(q);
}

static void ksw_ll_conf_run_case(int id, int size, int qlen, const uint8_t *query, int tlen,
                                 const uint8_t *target, int gapo, int gape, int xtra)
{
    ksw_ll_conf_run_case_mat(id, 0, size, qlen, query, tlen, target, gapo, gape, xtra);
}

int main(void)
{
    static const int xtras[] = {
        0,
        KSW_LL_SUBO | 8,
        KSW_LL_SUBO | 15,
        KSW_LL_STOP | 12,
        KSW_LL_STOP | 25,
        KSW_LL_SUBO | KSW_LL_STOP | 18,
        KSW_LL_SUBO | 1,
        KSW_LL_STOP | 1,
        KSW_LL_SUBO | KSW_LL_STOP | 1,
        KSW_LL_SUBO | 63,
        KSW_LL_STOP | 63,
        KSW_LL_SUBO | KSW_LL_STOP | 63,
    };
    static const int lens[] = {
        1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 31, 32, 33, 47, 48, 49, 63, 64, 65, 95, 96, 97, 127,
    };
    uint8_t query[160], target[160];
    uint32_t state = 0x11c0ffeeu;
    int id = 0, i, size;

    {
        static const uint8_t q0[] = {0, 1, 2, 3};
        static const uint8_t t0[] = {0, 1, 2, 3};
        static const uint8_t q1[] = {0, 1, 4, 3, 0, 2, 1, 1, 3, 4, 0, 2};
        static const uint8_t t1[] = {0, 4, 1, 3, 2, 1, 1, 3, 4, 0, 2};
        static const uint8_t q2[] = {4, 4, 4, 4, 0, 1, 2, 3, 4, 0, 2, 4, 1, 3, 4, 0, 1};
        static const uint8_t t2[] = {4, 0, 4, 1, 4, 2, 4, 3, 0, 4, 2, 1, 4, 3, 0, 4};
        for (size = 1; size <= 2; ++size) {
            ksw_ll_conf_run_case(id++, size, 4, q0, 4, t0, 5, 1, 0);
            ksw_ll_conf_run_case(id++, size, 12, q1, 11, t1, 4, 2, KSW_LL_SUBO | 6);
            ksw_ll_conf_run_case(id++, size, 12, q1, 11, t1, 4, 1, KSW_LL_STOP | 10);
            ksw_ll_conf_run_case(id++, size, 1, q2, 16, t2, 3, 1, KSW_LL_SUBO | 1);
            ksw_ll_conf_run_case(id++, size, 16, q2, 1, t2, 3, 1, KSW_LL_STOP | 2);
            ksw_ll_conf_run_case(id++, size, 17, q2, 16, t2, 6, 2,
                                 KSW_LL_SUBO | KSW_LL_STOP | 9);
        }
        for (i = 0; i < 127; ++i) {
            query[i] = 0;
            target[i] = 0;
        }
        for (size = 1; size <= 2; ++size) {
            ksw_ll_conf_run_case(id++, size, 80, query, 80, target, 5, 1, 0);
            ksw_ll_conf_run_case(id++, size, 127, query, 127, target, 5, 1, 0);
            ksw_ll_conf_run_case(id++, size, 127, query, 95, target, 4, 1, KSW_LL_SUBO | 30);
            ksw_ll_conf_run_case(id++, size, 95, query, 127, target, 4, 1, KSW_LL_STOP | 120);
        }
    }

    {
        static const int stress_xtras[] = {
            0,
            KSW_LL_SUBO | 4,
            KSW_LL_SUBO | 31,
            KSW_LL_STOP | 4,
            KSW_LL_STOP | 31,
            KSW_LL_SUBO | KSW_LL_STOP | 7,
            KSW_LL_SUBO | KSW_LL_STOP | 63,
            KSW_LL_SUBO | KSW_LL_STOP | 127,
        };
        int n_xtras = (int)(sizeof(stress_xtras) / sizeof(stress_xtras[0]));
        for (i = 0; i < 145; ++i) {
            query[i] = (uint8_t)((i / 4 + i / 13) % 4);
            target[i] = (uint8_t)((i / 4 + (i + 7) / 19) % 4);
            if (i % 29 == 0) query[i] = 4;
            if (i % 31 == 0) target[i] = 4;
        }
        for (size = 1; size <= 2; ++size) {
            for (i = 0; i < n_xtras; ++i) {
                int gapo = 4 + i % 5;
                int gape = 1 + i % 3;
                int xtra = stress_xtras[i];
                ksw_ll_conf_run_case_mat(id++, 2, size, 16, query, 16, target, gapo, gape,
                                         xtra);
                ksw_ll_conf_run_case_mat(id++, 2, size, 31, query + 1, 32, target + 2,
                                         gapo, gape, xtra);
                ksw_ll_conf_run_case_mat(id++, 2, size, 63, query + 3, 65, target + 4,
                                         gapo, gape, xtra);
                ksw_ll_conf_run_case_mat(id++, 2, size, 96, query, 95, target + 1, gapo,
                                         gape, xtra);
            }
        }
    }

    for (size = 1; size <= 2; ++size) {
        for (i = 0; i < (int)(sizeof(lens) / sizeof(lens[0])); ++i) {
            int qlen = lens[i];
            int tlen = lens[(i * 7 + 5) % (sizeof(lens) / sizeof(lens[0]))];
            int gapo = 3 + i % 7;
            int gape = 1 + (i * 3) % 4;
            int xtra = xtras[(i * 5 + size) % (sizeof(xtras) / sizeof(xtras[0]))];
            ksw_ll_conf_fill(query, qlen, &state);
            ksw_ll_conf_fill(target, tlen, &state);
            ksw_ll_conf_run_case(id++, size, qlen, query, tlen, target, gapo, gape, xtra);
        }
    }

    for (i = 0; i < 80; ++i) {
        int qlen = 1 + (int)(ksw_ll_conf_rand(&state) % 96u);
        int tlen = 1 + (int)(ksw_ll_conf_rand(&state) % 96u);
        int gapo = 3 + (int)(ksw_ll_conf_rand(&state) % 6u);
        int gape = 1 + (int)(ksw_ll_conf_rand(&state) % 3u);
        int xtra = xtras[ksw_ll_conf_rand(&state) % (sizeof(xtras) / sizeof(xtras[0]))];
        ksw_ll_conf_fill(query, qlen, &state);
        ksw_ll_conf_fill(target, tlen, &state);
        ksw_ll_conf_run_case_mat(id++, 1, (i & 1) + 1, qlen, query, tlen, target, gapo, gape,
                                 xtra);
    }

    for (i = 0; i < 240; ++i) {
        int qlen = 1 + (int)(ksw_ll_conf_rand(&state) % 127u);
        int tlen = 1 + (int)(ksw_ll_conf_rand(&state) % 127u);
        int gapo = 3 + (int)(ksw_ll_conf_rand(&state) % 7u);
        int gape = 1 + (int)(ksw_ll_conf_rand(&state) % 4u);
        int xtra = xtras[ksw_ll_conf_rand(&state) % (sizeof(xtras) / sizeof(xtras[0]))];
        ksw_ll_conf_fill(query, qlen, &state);
        ksw_ll_conf_fill(target, tlen, &state);
        ksw_ll_conf_run_case(id++, (i & 1) + 1, qlen, query, tlen, target, gapo, gape, xtra);
    }
    return 0;
}
