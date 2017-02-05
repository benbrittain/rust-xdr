enum geo_experiment_group_t {
    GEO_INVALID = 0,
    GEO_CONTROL = 1,
    GEO_EXPERIMENT = 2,
    GEO_NUM_GROUP_TYPES = 3,
};

enum cluster_type_t {
    CT_EXPERIMENT_CLUSTER_V0 = 0,
    CT_EXPERIMENT_CLUSTER_V1 = 1,
    CT_NUM_CLUSTER_TYPES = 2,
};

typedef int locid_t;
typedef locid_t locid_vec_t<>;

struct location_cluster_t {
    locid_t         locid;
    unsigned int    last_updated;
    unsigned int    cluster_id;
    cluster_type_t  cluster_type;
};

struct location_cluster_batch_arg_t {
    locid_vec_t         locs;
    location_cluster_t  cluster_arg;
};

union get_loc_cluster_res_t switch(geo_experiment_group_t status) {
    case GEO_INVALID:
        void;
    default:
        location_cluster_t loc_cluster;
};

typedef location_cluster_t location_cluster_vec_t<>;

// RPC Service code
program EXAMPLEDBD_PROG {
    version EXAMPLEDBD_VERS {
        void EXAMPLEDBD_NULL(void) = 0;
        get_loc_cluster_res_t GET_LOCATION_CLUSTER(location_cluster_t) = 1;
    } = 1;
} = 28755;
